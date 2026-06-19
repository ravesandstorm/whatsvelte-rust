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
// data dir) and rehydrate it on boot. Avatars are intentionally NOT cached —
// WhatsApp profile-picture URLs expire, so they are re-fetched live on demand.

import { chats, type Chat } from "./stores/chats.svelte";
import { messagesByChat, type UiMessage } from "./stores/messages.svelte";

const DB_NAME = "whatsvelte";
const DB_VERSION = 1;
const STORE_CHATS = "chats"; // keyPath: jid
const STORE_MESSAGES = "messages"; // keyPath: jid -> { jid, items }
const STORE_META = "meta"; // keyPath: k -> { k, v }
const META_ACCOUNT = "accountJid";

// We persist EVERY message and EVERY chat — the IndexedDB cache is the system of
// record for history (the backend exposes no query API). RAM is bounded on the
// rendering side (windowed message list), not here.
const DEBOUNCE_MS = 600;

let dbPromise: Promise<IDBDatabase> | null = null;

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
      if (!db.objectStoreNames.contains(STORE_CHATS))
        db.createObjectStore(STORE_CHATS, { keyPath: "jid" });
      if (!db.objectStoreNames.contains(STORE_MESSAGES))
        db.createObjectStore(STORE_MESSAGES, { keyPath: "jid" });
      if (!db.objectStoreNames.contains(STORE_META))
        db.createObjectStore(STORE_META, { keyPath: "k" });
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

function getAll<T>(store: string): Promise<T[]> {
  return openDb().then(
    (db) =>
      new Promise<T[]>((res, rej) => {
        const req = db.transaction([store], "readonly").objectStore(store).getAll();
        req.onsuccess = () => res(req.result as T[]);
        req.onerror = () => rej(req.error);
      }),
  );
}

function getMeta<T>(k: string): Promise<T | null> {
  return openDb().then(
    (db) =>
      new Promise<T | null>((res, rej) => {
        const req = db.transaction([STORE_META], "readonly").objectStore(STORE_META).get(k);
        req.onsuccess = () => res(req.result ? ((req.result as { v: T }).v ?? null) : null);
        req.onerror = () => rej(req.error);
      }),
  );
}

export interface Snapshot {
  accountJid: string | null;
  chats: Chat[];
  messages: Record<string, UiMessage[]>;
}

/** Load everything persisted for the previous run. */
export async function loadSnapshot(): Promise<Snapshot> {
  const [accountJid, chatRows, msgRows] = await Promise.all([
    getMeta<string>(META_ACCOUNT),
    getAll<Chat>(STORE_CHATS),
    getAll<{ jid: string; items: UiMessage[] }>(STORE_MESSAGES),
  ]);
  const messages: Record<string, UiMessage[]> = {};
  for (const r of msgRows) messages[r.jid] = r.items;
  return { accountJid, chats: chatRows, messages };
}

/** Wipe the cache (logout, or a different account was linked). */
export async function clearAll(): Promise<void> {
  const db = await openDb();
  const t = db.transaction([STORE_CHATS, STORE_MESSAGES, STORE_META], "readwrite");
  t.objectStore(STORE_CHATS).clear();
  t.objectStore(STORE_MESSAGES).clear();
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

// --- Debounced mirroring of the live stores -------------------------------

// Persistence stays off until boot hydration has run, so rehydrating the stores
// doesn't immediately race a write back to disk.
let enabled = false;
let chatsTimer: ReturnType<typeof setTimeout> | null = null;
let msgTimer: ReturnType<typeof setTimeout> | null = null;
const dirtyMsgJids = new Set<string>();

export function enablePersistence() {
  enabled = true;
}

export function schedulePersistChats() {
  if (!enabled || chatsTimer) return;
  chatsTimer = setTimeout(() => {
    chatsTimer = null;
    void flushChats();
  }, DEBOUNCE_MS);
}

export function schedulePersistMessages(jid: string) {
  if (!enabled) return;
  dirtyMsgJids.add(jid);
  if (msgTimer) return;
  msgTimer = setTimeout(() => {
    msgTimer = null;
    void flushMessages();
  }, DEBOUNCE_MS);
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

async function flushMessages() {
  const jids = [...dirtyMsgJids];
  dirtyMsgJids.clear();
  try {
    const db = await openDb();
    const t = db.transaction([STORE_MESSAGES], "readwrite");
    const store = t.objectStore(STORE_MESSAGES);
    for (const jid of jids) {
      const items = messagesByChat.get(jid);
      if (!items || items.length === 0) continue;
      // Store every message; only normalize a still-"sending" optimistic bubble
      // so it doesn't resurrect as permanently pending on reload.
      const persisted = items.map((m) =>
        m.status === "sending" ? { ...m, status: "sent" as const } : m,
      );
      store.put({ jid, items: persisted });
    }
    await txDone(t);
  } catch (e) {
    console.error("persist messages failed", e);
  }
}
