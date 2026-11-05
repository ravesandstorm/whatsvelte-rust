// Frontend-side persistence of chat/message state in IndexedDB.
//
// The Rust backend persists the WhatsApp *session* (its SqliteStore), but it
// exposes no query API for past chats/messages — those reach us only as events:
// a one-time `HistorySync` at pairing, then live `Message` events. On a plain
// app restart the session is already stored, so *no* HistorySync fires and the
// purely event-derived, in-memory UI would start empty.
//
// To fix that without touching the backend, we mirror the event-derived state
// into the webview's IndexedDB (persistent across restarts, lives in the app's
// data dir) and page it back in on demand. Avatars are intentionally NOT cached
// — WhatsApp profile-picture URLs expire, so they are re-fetched live.
//
// Memory model: IndexedDB is the system of record for message history. Each
// message is its own record (keyed by [chatJid, id]) so a change writes one
// small row instead of re-serializing the whole chat, and the in-memory store
// only keeps a small resident window per chat (see stores/messages.svelte.ts),
// paging older messages in from here when the user scrolls.

import { chats, type Chat } from "./stores/chats.svelte";
import type { UiMessage } from "./stores/messages.svelte";

const DB_NAME = "whatsvelte";
// v1: one `messages` row per chat ({ jid, items: UiMessage[] }).
// v2: one `msgs` row per message, migrated in place from v1.
const DB_VERSION = 2;
const STORE_CHATS = "chats"; // keyPath: jid
const STORE_MSGS = "msgs"; // keyPath: [chatJid, id]
const IDX_CHAT_TIME = "byChatTime"; // [chatJid, timestamp]
const STORE_META = "meta"; // keyPath: k -> { k, v }
const LEGACY_MESSAGES = "messages"; // v1 store, dropped after migration
const META_ACCOUNT = "accountJid";

const DEBOUNCE_MS = 600;
/** Max records per write transaction, so a big history-sync burst is committed
 * in bounded slices instead of one giant transaction buffered in memory. */
const FLUSH_SLICE = 1000;

let dbPromise: Promise<IDBDatabase> | null = null;

/** Whether a message should be persisted at all. Optimistic bubbles (`tmp-`
 * ids) are skipped: they are persisted once the server assigns a real id, and a
 * send that never confirms simply doesn't survive a restart (rather than
 * resurrecting as a falsely "sent" message). */
function persistable(m: UiMessage): boolean {
  return !!m.id && !!m.chatJid && !m.id.startsWith("tmp-") && m.status !== "sending";
}

/** Normalize a message into a storable record (valid index keys). */
function toRecord(m: UiMessage, chatJid = m.chatJid): UiMessage {
  const timestamp = Number(m.timestamp);
  return {
    ...m,
    chatJid,
    timestamp: Number.isFinite(timestamp) ? timestamp : 0,
  };
}

/** Copy every v1 per-chat row into per-message records inside the upgrade
 * transaction, creating a minimal chat row when one is missing, then drop the
 * v1 store. Cursor-driven, so only one legacy chat row is in memory at a time. */
function migrateLegacy(db: IDBDatabase, tx: IDBTransaction) {
  const legacy = tx.objectStore(LEGACY_MESSAGES);
  const msgs = tx.objectStore(STORE_MSGS);
  const chatStore = tx.objectStore(STORE_CHATS);
  const cursorReq = legacy.openCursor();
  cursorReq.onsuccess = () => {
    const cursor = cursorReq.result;
    if (!cursor) {
      db.deleteObjectStore(LEGACY_MESSAGES);
      return;
    }
    const row = cursor.value as { jid: string; items?: UiMessage[] };
    const items = row.items ?? [];
    for (const m of items) {
      if (persistable(m)) msgs.put(toRecord(m, row.jid));
    }
    const last = items[items.length - 1];
    if (last) {
      const existing = chatStore.get(row.jid);
      existing.onsuccess = () => {
        if (existing.result) return;
        const chat: Chat = {
          jid: row.jid,
          name: last.pushName ?? null,
          lastMessage: last.text ?? null,
          timestamp: last.timestamp ?? 0,
          unread: 0,
        };
        chatStore.put(chat);
      };
    }
    cursor.continue();
  };
}

function openDb(): Promise<IDBDatabase> {
  if (dbPromise) return dbPromise;
  dbPromise = new Promise((resolve, reject) => {
    if (typeof indexedDB === "undefined") {
      reject(new Error("IndexedDB unavailable"));
      return;
    }
    const req = indexedDB.open(DB_NAME, DB_VERSION);
    req.onupgradeneeded = () => {
      const db = req.result;
      const tx = req.transaction!;
      if (!db.objectStoreNames.contains(STORE_CHATS))
        db.createObjectStore(STORE_CHATS, { keyPath: "jid" });
      if (!db.objectStoreNames.contains(STORE_META))
        db.createObjectStore(STORE_META, { keyPath: "k" });
      if (!db.objectStoreNames.contains(STORE_MSGS)) {
        const msgs = db.createObjectStore(STORE_MSGS, { keyPath: ["chatJid", "id"] });
        msgs.createIndex(IDX_CHAT_TIME, ["chatJid", "timestamp"]);
      }
      if (db.objectStoreNames.contains(LEGACY_MESSAGES)) migrateLegacy(db, tx);
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
  return dbPromise;
}

function txDone(t: IDBTransaction): Promise<void> {
  return new Promise((res, rej) => {
    t.oncomplete = () => res();
    t.onerror = () => rej(t.error);
    t.onabort = () => rej(t.error);
  });
}

function reqResult<T>(r: IDBRequest<T>): Promise<T> {
  return new Promise((res, rej) => {
    r.onsuccess = () => res(r.result);
    r.onerror = () => rej(r.error);
  });
}

/** Primary-key range covering every message of one chat. Arrays sort after
 * strings, so `[jid, []]` is above every `[jid, "<id>"]`. */
function chatKeyRange(jid: string): IDBKeyRange {
  return IDBKeyRange.bound([jid], [jid, []]);
}

export interface Snapshot {
  accountJid: string | null;
  chats: Chat[];
  /** Whether any message is persisted (used to detect stale data on logout). */
  hasMessages: boolean;
}

/** Load the chat rows + account marker. Messages are NOT loaded here — they are
 * paged in per chat via `loadLatestMessages` / `loadTails`. */
export async function loadSnapshot(): Promise<Snapshot> {
  const db = await openDb();
  const t = db.transaction([STORE_META, STORE_CHATS, STORE_MSGS], "readonly");
  const [meta, chatRows, msgCount] = await Promise.all([
    reqResult(t.objectStore(STORE_META).get(META_ACCOUNT)),
    reqResult(t.objectStore(STORE_CHATS).getAll()),
    reqResult(t.objectStore(STORE_MSGS).count()),
  ]);
  return {
    accountJid: (meta as { v: string | null } | undefined)?.v ?? null,
    chats: chatRows as Chat[],
    hasMessages: msgCount > 0,
  };
}

/** Collect up to `limit` newest messages of `jid` from an index cursor, with
 * timestamp <= `beforeTs` when given. Returns them oldest-first. */
function readLatest(
  index: IDBIndex,
  jid: string,
  limit: number,
  beforeTs: number | undefined,
): Promise<UiMessage[]> {
  const upper = beforeTs === undefined ? [jid, Infinity] : [jid, beforeTs];
  const range = IDBKeyRange.bound([jid, -Infinity], upper);
  return new Promise((res, rej) => {
    const out: UiMessage[] = [];
    if (limit <= 0) {
      res(out);
      return;
    }
    const req = index.openCursor(range, "prev");
    req.onsuccess = () => {
      const cursor = req.result;
      if (!cursor || out.length >= limit) {
        res(out.reverse());
        return;
      }
      out.push(cursor.value as UiMessage);
      cursor.continue();
    };
    req.onerror = () => rej(req.error);
  });
}

/** Unflushed puts for `jid` (they aren't in IndexedDB yet but may already have
 * been trimmed out of memory). */
function pendingFor(jid: string, beforeTs: number | undefined): UiMessage[] {
  const out: UiMessage[] = [];
  for (const op of pending.values()) {
    if (op.op === "del" || op.m.chatJid !== jid) continue;
    if (beforeTs !== undefined && op.m.timestamp > beforeTs) continue;
    out.push(op.m);
  }
  return out;
}

/** Merge persisted rows with unflushed puts (pending wins), keep the newest
 * `limit`, oldest-first. */
function mergePending(rows: UiMessage[], extra: UiMessage[], limit: number): UiMessage[] {
  if (!extra.length) return rows;
  const byId = new Map<string, UiMessage>();
  for (const m of rows) byId.set(m.id, m);
  for (const m of extra) byId.set(m.id, m);
  const all = [...byId.values()].sort((a, b) => a.timestamp - b.timestamp);
  return all.slice(Math.max(0, all.length - limit));
}

/** The newest `limit` messages of a chat, oldest-first. With `beforeTs`, only
 * messages at or before that second (inclusive, so same-second neighbours of
 * the current oldest message aren't skipped — callers dedupe by id). */
export async function loadLatestMessages(
  jid: string,
  limit: number,
  beforeTs?: number,
): Promise<UiMessage[]> {
  const db = await openDb();
  const index = db.transaction([STORE_MSGS], "readonly").objectStore(STORE_MSGS).index(IDX_CHAT_TIME);
  const rows = await readLatest(index, jid, limit, beforeTs);
  return mergePending(rows, pendingFor(jid, beforeTs), limit);
}

/** The newest `n` messages of each chat, read in a single transaction (boot). */
export async function loadTails(jids: string[], n: number): Promise<Map<string, UiMessage[]>> {
  const out = new Map<string, UiMessage[]>();
  if (!jids.length || n <= 0) return out;
  const db = await openDb();
  const index = db.transaction([STORE_MSGS], "readonly").objectStore(STORE_MSGS).index(IDX_CHAT_TIME);
  const results = await Promise.all(jids.map((jid) => readLatest(index, jid, n, undefined)));
  jids.forEach((jid, i) => {
    const rows = mergePending(results[i], pendingFor(jid, undefined), n);
    if (rows.length) out.set(jid, rows);
  });
  return out;
}

/** Move a chat's persisted messages (and chat row) from one key to another —
 * LID→PN unification. Pending writes for the old key are re-keyed too. */
export async function movePersistedChat(fromJid: string, toJid: string): Promise<void> {
  if (fromJid === toJid) return;
  for (const [k, op] of [...pending]) {
    if (op.op !== "del" && op.m.chatJid === fromJid) {
      pending.delete(k);
      const m = { ...op.m, chatJid: toJid };
      pending.set(pendingKey(toJid, m.id), { op: op.op, m });
    } else if (op.op === "del" && op.chatJid === fromJid) {
      pending.delete(k);
    }
  }
  try {
    const db = await openDb();
    const t = db.transaction([STORE_CHATS, STORE_MSGS], "readwrite");
    const msgs = t.objectStore(STORE_MSGS);
    const cursorReq = msgs.openCursor(chatKeyRange(fromJid));
    cursorReq.onsuccess = () => {
      const cursor = cursorReq.result;
      if (!cursor) return;
      msgs.put({ ...(cursor.value as UiMessage), chatJid: toJid });
      cursor.delete();
      cursor.continue();
    };
    t.objectStore(STORE_CHATS).delete(fromJid);
    await txDone(t);
  } catch (e) {
    console.error("move persisted chat failed", e);
  }
}

/** Wipe the cache (logout, or a different account was linked). */
export async function clearAll(): Promise<void> {
  const db = await openDb();
  const t = db.transaction([STORE_CHATS, STORE_MSGS, STORE_META], "readwrite");
  t.objectStore(STORE_CHATS).clear();
  t.objectStore(STORE_MSGS).clear();
  t.objectStore(STORE_META).clear();
  await txDone(t);
}

/** Record which account this cache belongs to, so a re-link can invalidate it. */
export async function setAccount(jid: string | null): Promise<void> {
  try {
    const db = await openDb();
    const t = db.transaction([STORE_META], "readwrite");
    t.objectStore(STORE_META).put({ k: META_ACCOUNT, v: jid });
    await txDone(t);
  } catch (e) {
    console.error("persist account failed", e);
  }
}

// --- Debounced per-message writes -----------------------------------------

// "put" replaces the stored record; "add" inserts only if no record exists
// (new arrivals, so a redelivered old message can't clobber its stored copy).
type PendingOp =
  | { op: "put"; m: UiMessage }
  | { op: "add"; m: UiMessage }
  | { op: "del"; chatJid: string; id: string };

// Writes are queued even while persistence is disabled (boot hydration), so a
// live message that arrives mid-hydration — and may be trimmed out of memory
// before persistence turns on — is still written once it is enabled.
const pending = new Map<string, PendingOp>();
let enabled = false;
let chatsTimer: ReturnType<typeof setTimeout> | null = null;
let msgTimer: ReturnType<typeof setTimeout> | null = null;

function pendingKey(chatJid: string, id: string): string {
  return chatJid + "\u0001" + id;
}

export function enablePersistence() {
  enabled = true;
  schedulePersistChats();
  scheduleFlushMessages();
}

/** Drop queued, unwritten message ops (full reset). */
export function discardPending() {
  pending.clear();
}

export function schedulePersistChats() {
  if (!enabled || chatsTimer) return;
  chatsTimer = setTimeout(() => {
    chatsTimer = null;
    void flushChats();
  }, DEBOUNCE_MS);
}

function scheduleFlushMessages() {
  if (!enabled || msgTimer || pending.size === 0) return;
  msgTimer = setTimeout(() => {
    msgTimer = null;
    void flushMessages();
  }, DEBOUNCE_MS);
}

/** Queue a write of one message (insert or update). */
export function persistMessage(m: UiMessage) {
  if (!persistable(m)) return;
  pending.set(pendingKey(m.chatJid, m.id), { op: "put", m });
  scheduleFlushMessages();
}

/** Queue an insert-only write: a no-op if the message is already stored. */
export function persistMessageInsert(m: UiMessage) {
  if (!persistable(m)) return;
  const k = pendingKey(m.chatJid, m.id);
  if (pending.get(k)?.op === "put") return; // a full write is already queued
  pending.set(k, { op: "add", m });
  scheduleFlushMessages();
}

/** The stored copy of a message (including a queued full write), if any. */
export async function getPersistedMessage(
  chatJid: string,
  id: string,
): Promise<UiMessage | undefined> {
  const queued = pending.get(pendingKey(chatJid, id));
  if (queued?.op === "put") return queued.m;
  if (queued?.op === "del") return undefined;
  try {
    const db = await openDb();
    const req = db.transaction([STORE_MSGS], "readonly").objectStore(STORE_MSGS).get([chatJid, id]);
    return (await reqResult(req)) as UiMessage | undefined;
  } catch (e) {
    console.error("read persisted message failed", e);
    return undefined;
  }
}

/** Queue a delete of one message. */
export function persistMessageDelete(chatJid: string, id: string) {
  if (!id) return;
  pending.set(pendingKey(chatJid, id), { op: "del", chatJid, id });
  scheduleFlushMessages();
}

/**
 * Update a message that is not resident in memory: applies `fn` to the queued
 * write if one is pending, otherwise read-modify-writes the persisted record.
 * `fn` returning null leaves the record unchanged. Resolves to the updated
 * message, or undefined when no such message is persisted.
 */
export async function patchPersistedMessage(
  chatJid: string,
  id: string,
  fn: (m: UiMessage) => UiMessage | null,
): Promise<UiMessage | undefined> {
  const k = pendingKey(chatJid, id);
  const queued = pending.get(k);
  if (queued?.op === "put") {
    const next = fn(queued.m);
    if (next) pending.set(k, { op: "put", m: next });
    return next ?? queued.m;
  }
  if (queued?.op === "del") return undefined;
  try {
    const db = await openDb();
    const t = db.transaction([STORE_MSGS], "readwrite");
    const done = txDone(t);
    const store = t.objectStore(STORE_MSGS);
    // The put is issued from the get's promise continuation, which runs while
    // the transaction is still active.
    const stored = (await reqResult(store.get([chatJid, id]))) as UiMessage | undefined;
    // A queued insert-only write only matters when nothing is stored yet.
    const current = stored ?? (queued?.op === "add" ? queued.m : undefined);
    if (stored && queued?.op === "add") pending.delete(k);
    if (!current) {
      await done;
      return undefined;
    }
    const next = fn(current);
    if (next) store.put(toRecord(next));
    await done;
    return next ?? current;
  } catch (e) {
    console.error("patch persisted message failed", e);
    return undefined;
  }
}

async function flushChats() {
  try {
    const db = await openDb();
    const t = db.transaction([STORE_CHATS], "readwrite");
    const store = t.objectStore(STORE_CHATS);
    // Additive put per chat — never clear(). An empty/partly-hydrated in-memory
    // map must never be able to wipe the persisted chats (only logout, via
    // clearAll, removes them).
    for (const c of chats.values()) store.put({ ...c });
    await txDone(t);
  } catch (e) {
    console.error("persist chats failed", e);
  }
}

/** Write queued message ops now instead of waiting for the debounce. */
export async function flushPendingWrites(): Promise<void> {
  if (msgTimer) {
    clearTimeout(msgTimer);
    msgTimer = null;
  }
  await flushMessages();
}

async function flushMessages() {
  const ops = [...pending.values()];
  pending.clear();
  try {
    const db = await openDb();
    for (let i = 0; i < ops.length; i += FLUSH_SLICE) {
      const t = db.transaction([STORE_MSGS], "readwrite");
      const store = t.objectStore(STORE_MSGS);
      for (const op of ops.slice(i, i + FLUSH_SLICE)) {
        if (op.op === "put") store.put(toRecord(op.m));
        else if (op.op === "add") {
          // Already stored → ConstraintError; swallow it so the rest of the
          // transaction still commits.
          const req = store.add(toRecord(op.m));
          req.onerror = (ev) => {
            ev.preventDefault();
            ev.stopPropagation();
          };
        } else store.delete([op.chatJid, op.id]);
      }
      await txDone(t);
    }
  } catch (e) {
    console.error("persist messages failed", e);
  }
  // Writes queued while this flush was running.
  scheduleFlushMessages();
}
