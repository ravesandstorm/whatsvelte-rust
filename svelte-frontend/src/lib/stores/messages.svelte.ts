// Per-chat message lists — a bounded, in-memory window over IndexedDB.
//
// IndexedDB (lib/persist.ts) holds the full history. Memory holds only:
//   - the open chat: whatever the user has paged in (reset when they leave), and
//   - every other chat: its newest RESIDENT_TAIL messages (list preview, ticks).
// Older messages are paged back in on scroll via `loadOlder`. Mutations to a
// message that is no longer resident are applied straight to IndexedDB.

import { SvelteMap, SvelteSet } from "svelte/reactivity";
import type { MessageDto, QuotedDto } from "../types";
import {
  loadLatestMessages,
  patchPersistedMessage,
  persistMessage,
  persistMessageDelete,
} from "../persist";

export type MessageStatus = "sending" | "sent" | "delivered" | "read" | "played";

export interface UiMessage extends MessageDto {
  status?: MessageStatus;
  /** Set when the message was edited (epoch secs of the edit). */
  editedAt?: number;
  /** True when the message was deleted/revoked for everyone. */
  deleted?: boolean;
  /** Emoji reactions keyed by reactor JID ("" = none). */
  reactions?: Record<string, string>;
  /** True if this incoming message hasn't been scrolled into view / read yet. */
  unread?: boolean;
}

import { chats, chatUi, decrementUnread } from "./chats.svelte";

/** Messages kept in memory for chats that aren't open. */
export const RESIDENT_TAIL = 20;
/** Messages paged in per scroll-up / on open. */
export const PAGE = 50;
/** Upper bound on the initial load when a chat has many unread messages. */
const MAX_INITIAL = 500;

// Monotonic ordering so a later out-of-order receipt can't downgrade status.
const STATUS_RANK: Record<MessageStatus, number> = {
  sending: 0,
  sent: 1,
  delivered: 2,
  read: 3,
  played: 4,
};

export const messagesByChat = new SvelteMap<string, UiMessage[]>();
/** Chats whose full persisted history is already in memory (nothing older to
 * page in). Reactive so the list's "load older" affordance updates. */
const exhausted = new SvelteSet<string>();
/** In-flight `loadOlder` per chat, so concurrent scroll events share one read. */
const loadingOlder = new Map<string, Promise<number>>();

function ensure(jid: string): UiMessage[] {
  let arr = messagesByChat.get(jid);
  if (!arr) {
    arr = [];
    messagesByChat.set(jid, arr);
  }
  return arr;
}

/** Sort, trim to the resident tail when the chat isn't open, and publish a new
 * array reference so SvelteMap consumers re-render. Persistence is explicit
 * (callers `persistMessage` exactly what they changed). */
function commit(jid: string, arr: UiMessage[]) {
  arr.sort((a, b) => a.timestamp - b.timestamp);
  if (chatUi.activeJid !== jid && arr.length > RESIDENT_TAIL) {
    messagesByChat.set(jid, arr.slice(arr.length - RESIDENT_TAIL));
    exhausted.delete(jid);
  } else {
    messagesByChat.set(jid, [...arr]);
  }
}

/** Flag the last `chat.unread` incoming messages of `arr` as unread (and the
 * rest as read), in place. Returns the messages that changed. */
function applyUnreadFlags(jid: string, arr: UiMessage[]): UiMessage[] {
  const count = chats.get(jid)?.unread ?? 0;
  const unreadIdx = new Set<number>();
  for (let i = arr.length - 1; i >= 0 && unreadIdx.size < count; i--) {
    if (!arr[i].fromMe) unreadIdx.add(i);
  }
  const changed: UiMessage[] = [];
  for (let i = 0; i < arr.length; i++) {
    const shouldBeUnread = unreadIdx.has(i);
    if (!!arr[i].unread !== shouldBeUnread) {
      arr[i] = { ...arr[i], unread: shouldBeUnread };
      changed.push(arr[i]);
    }
  }
  return changed;
}

/** Ensure the last `unreadCount` incoming messages in `jid` are flagged unread. */
export function syncUnreadFlagForChat(jid: string) {
  const arr = messagesByChat.get(jid);
  if (!arr || arr.length === 0) return;
  const changed = applyUnreadFlags(jid, arr);
  if (!changed.length) return;
  changed.forEach(persistMessage);
  commit(jid, arr);
}

export function addMessage(m: UiMessage) {
  const arr = ensure(m.chatJid);

  // Incoming messages start unread by default unless explicitly marked read
  const msgToAdd: UiMessage =
    !m.fromMe && m.unread === undefined ? { ...m, unread: true } : m;

  if (msgToAdd.id) {
    const dupIdx = arr.findIndex((x) => x.id === msgToAdd.id);
    if (dupIdx >= 0) {
      const existing = arr[dupIdx];
      if (msgToAdd.media && !existing.media) {
        arr[dupIdx] = { ...existing, media: msgToAdd.media, thumbnail: msgToAdd.thumbnail ?? existing.thumbnail };
        persistMessage(arr[dupIdx]);
        commit(msgToAdd.chatJid, arr);
      }
      return;
    }
  }

  // Reconcile the server echo of our own send with the optimistic bubble.
  if (msgToAdd.fromMe) {
    const tmp = arr.findIndex((x) => x.status === "sending" && x.text === msgToAdd.text);
    if (tmp >= 0) {
      arr[tmp] = { ...msgToAdd, status: "sent" };
      persistMessage(arr[tmp]);
      commit(msgToAdd.chatJid, arr);
      return;
    }
  }

  arr.push(msgToAdd);
  persistMessage(msgToAdd);
  commit(msgToAdd.chatJid, arr);
}

/**
 * Bulk insert (history sync, or rows read back from IndexedDB). Dedupes by id.
 * `persist: false` is for rows that came *from* IndexedDB, so they aren't
 * written straight back.
 */
export function addHistoryMessages(list: MessageDto[], opts: { persist?: boolean } = {}) {
  const persist = opts.persist ?? true;
  const touched = new Map<string, Set<string>>();
  for (const m of list) {
    const arr = ensure(m.chatJid);
    let seen = touched.get(m.chatJid);
    if (!seen) {
      seen = new Set(arr.map((x) => x.id));
      touched.set(m.chatJid, seen);
    }
    if (m.id && seen.has(m.id)) continue;
    arr.push(m);
    if (m.id) seen.add(m.id);
    if (persist) persistMessage(m);
  }
  for (const jid of touched.keys()) {
    const arr = messagesByChat.get(jid)!;
    arr.sort((a, b) => a.timestamp - b.timestamp);
    // Flag unread on the full batch *before* trimming, so the flags are
    // persisted even for messages that don't stay resident.
    applyUnreadFlags(jid, arr).forEach(persistMessage);
    commit(jid, arr);
  }
}

/** Mark a single message read and decrement the chat's unread counter. */
export function markMessageRead(jid: string, messageId: string) {
  const arr = messagesByChat.get(jid);
  if (!arr) return;
  const idx = arr.findIndex((m) => m.id === messageId);
  if (idx >= 0 && arr[idx].unread) {
    arr[idx] = { ...arr[idx], unread: false };
    persistMessage(arr[idx]);
    commit(jid, arr);
    decrementUnread(jid, 1);
  }
}

/** Mark multiple messages read and decrement the chat's unread counter. */
export function markMessagesRead(jid: string, messageIds: string[]) {
  const arr = messagesByChat.get(jid);
  if (!arr) return;
  const idSet = new Set(messageIds);
  let readCount = 0;
  for (let i = 0; i < arr.length; i++) {
    if (idSet.has(arr[i].id) && arr[i].unread) {
      arr[i] = { ...arr[i], unread: false };
      persistMessage(arr[i]);
      readCount++;
    }
  }
  if (readCount > 0) {
    commit(jid, arr);
    decrementUnread(jid, readCount);
  }
}

export function addOptimistic(jid: string, text: string, quoted: QuotedDto | null = null): string {
  const tempId = "tmp-" + Math.random().toString(36).slice(2);
  addMessage({
    id: tempId,
    chatJid: jid,
    senderJid: "",
    fromMe: true,
    timestamp: Math.floor(Date.now() / 1000),
    pushName: null,
    text,
    kind: "text",
    thumbnail: null,
    media: null,
    quoted,
    status: "sending",
  });
  return tempId;
}

/** Optimistic bubble for an outgoing media send. `thumbnail` (base64 JPEG) gives
 * an instant preview for images/videos; `text` carries the caption or a label
 * (e.g. "📄 file.pdf") for non-thumbnail kinds. Confirmed via `confirmOptimistic`. */
export function addOptimisticMedia(
  jid: string,
  kind: string,
  text: string | null,
  thumbnail: string | null,
): string {
  const tempId = "tmp-" + Math.random().toString(36).slice(2);
  addMessage({
    id: tempId,
    chatJid: jid,
    senderJid: "",
    fromMe: true,
    timestamp: Math.floor(Date.now() / 1000),
    pushName: null,
    text,
    kind,
    thumbnail,
    media: null,
    quoted: null,
    status: "sending",
  });
  return tempId;
}

export function confirmOptimistic(jid: string, tempId: string, realId: string) {
  const arr = messagesByChat.get(jid);
  if (!arr) return;
  const idx = arr.findIndex((x) => x.id === tempId);
  if (idx >= 0) {
    arr[idx] = { ...arr[idx], id: realId, status: "sent" };
    persistMessage(arr[idx]);
    commit(jid, arr);
  }
}

export function messagesFor(jid: string): UiMessage[] {
  return messagesByChat.get(jid) ?? [];
}

/** Whether older messages may exist in IndexedDB beyond what's in memory. */
export function hasOlder(jid: string): boolean {
  return !exhausted.has(jid);
}

/**
 * Page up to `count` older messages of `jid` in from IndexedDB, prepending them
 * to the in-memory list. Resolves to the number of messages added. The window
 * is not trimmed here — callers page into the open chat.
 */
export function loadOlder(jid: string, count = PAGE): Promise<number> {
  const inflight = loadingOlder.get(jid);
  if (inflight) return inflight;
  const p = (async () => {
    if (exhausted.has(jid)) return 0;
    const before = messagesByChat.get(jid) ?? [];
    // The read is inclusive of the oldest resident second, so over-fetch by the
    // number of resident messages sharing it; that guarantees progress even
    // when many messages share one timestamp.
    const oldestTs = before.length ? before[0].timestamp : undefined;
    const sameSecond =
      oldestTs === undefined ? 0 : before.filter((m) => m.timestamp === oldestTs).length;
    const limit = count + sameSecond;
    const rows = await loadLatestMessages(jid, limit, oldestTs);
    if (rows.length < limit) exhausted.add(jid);

    const cur = messagesByChat.get(jid) ?? [];
    const ids = new Set(cur.map((m) => m.id));
    const fresh = rows.filter((m) => !ids.has(m.id));
    if (fresh.length) {
      const next = [...fresh, ...cur].sort((a, b) => a.timestamp - b.timestamp);
      messagesByChat.set(jid, next);
    }
    return fresh.length;
  })().finally(() => loadingOlder.delete(jid));
  loadingOlder.set(jid, p);
  return p;
}

/** Page older history in until message `id` is resident (or history runs out /
 * `maxPages` is hit). Resolves to whether it was found. */
export async function loadUntil(jid: string, id: string, maxPages = 40): Promise<boolean> {
  for (let i = 0; i < maxPages; i++) {
    if (messagesFor(jid).some((m) => m.id === id)) return true;
    if (!hasOlder(jid)) return false;
    await loadOlder(jid, PAGE);
  }
  return messagesFor(jid).some((m) => m.id === id);
}

/** A chat was opened: page in enough history to show the unread messages (at
 * least one page), then re-derive unread flags over what's resident. */
export async function openChat(jid: string): Promise<void> {
  const unread = chats.get(jid)?.unread ?? 0;
  const want = Math.min(MAX_INITIAL, Math.max(PAGE, unread + 10));
  const have = messagesFor(jid).length;
  if (have < want && hasOlder(jid)) await loadOlder(jid, want - have);
  syncUnreadFlagForChat(jid);
}

/** A chat was closed: drop its paged-in history back to the resident tail. */
export function closeChat(jid: string) {
  if (chatUi.activeJid === jid) return; // re-selected the same chat
  const arr = messagesByChat.get(jid);
  if (arr && arr.length > RESIDENT_TAIL) {
    messagesByChat.set(jid, arr.slice(arr.length - RESIDENT_TAIL));
    exhausted.delete(jid);
  }
}

/** Forget everything in memory (logout / reset). */
export function clearMessages() {
  messagesByChat.clear();
  exhausted.clear();
  loadingOlder.clear();
}

/** Move messages from one chat key into another (LID→PN unification), deduping
 * by id. Removes the source key from memory; the caller moves the persisted
 * rows (`movePersistedChat`). */
export function mergeMessages(fromJid: string, toJid: string) {
  const from = messagesByChat.get(fromJid);
  messagesByChat.delete(fromJid);
  exhausted.delete(fromJid);
  if (!from || from.length === 0) return;
  const to = messagesByChat.get(toJid) ?? [];
  const seen = new Set(to.map((m) => m.id));
  for (const m of from) {
    if (m.id && seen.has(m.id)) continue;
    to.push({ ...m, chatJid: toJid });
    if (m.id) seen.add(m.id);
  }
  // The merged-in rows may predate what's resident for `toJid`.
  exhausted.delete(toJid);
  commit(toJid, to);
}

/** Mark a message deleted-for-everyone, inserting a tombstone if unseen. */
export function applyRevoke(
  jid: string,
  targetId: string,
  fallback?: { senderJid: string; fromMe: boolean; timestamp: number },
) {
  const revoke = (m: UiMessage): UiMessage => ({
    ...m,
    deleted: true,
    text: null,
    thumbnail: null,
    reactions: undefined,
  });
  const arr = messagesByChat.get(jid);
  const idx = arr ? arr.findIndex((m) => m.id === targetId) : -1;
  if (arr && idx >= 0) {
    arr[idx] = revoke(arr[idx]);
    persistMessage(arr[idx]);
    commit(jid, arr);
    return;
  }
  void (async () => {
    const patched = await patchPersistedMessage(jid, targetId, revoke);
    if (patched || !fallback) return;
    const tombstone: UiMessage = {
      id: targetId,
      chatJid: jid,
      senderJid: fallback.senderJid,
      fromMe: fallback.fromMe,
      timestamp: fallback.timestamp,
      pushName: null,
      text: null,
      kind: "other",
      thumbnail: null,
      media: null,
      quoted: null,
      deleted: true,
    };
    const cur = ensure(jid);
    if (cur.some((m) => m.id === targetId)) return; // arrived meanwhile
    cur.push(tombstone);
    persistMessage(tombstone);
    commit(jid, cur);
  })();
}

/** Remove a message entirely (delete-for-me — no tombstone, unlike revoke). */
export function removeMessage(jid: string, targetId: string) {
  persistMessageDelete(jid, targetId);
  const arr = messagesByChat.get(jid);
  if (!arr) return;
  const next = arr.filter((m) => m.id !== targetId);
  if (next.length === arr.length) return;
  messagesByChat.set(jid, next);
}

/** Apply `fn` to message `targetId` — in memory when resident, otherwise to the
 * persisted record. `fn` returning null means "no change". */
function updateMessage(jid: string, targetId: string, fn: (m: UiMessage) => UiMessage | null) {
  const arr = messagesByChat.get(jid);
  const idx = arr ? arr.findIndex((m) => m.id === targetId) : -1;
  if (arr && idx >= 0) {
    const next = fn(arr[idx]);
    if (!next) return;
    arr[idx] = next;
    persistMessage(next);
    messagesByChat.set(jid, [...arr]);
    return;
  }
  void patchPersistedMessage(jid, targetId, fn);
}

/** Replace a message's text with its edited content. */
export function applyEdit(jid: string, targetId: string, text: string | null, editedAt: number) {
  updateMessage(jid, targetId, (m) => ({ ...m, text, editedAt }));
}

/** Add/remove a reaction (empty emoji removes the reactor's reaction). */
export function applyReaction(
  jid: string,
  targetId: string,
  reactor: string,
  emoji: string,
) {
  updateMessage(jid, targetId, (m) => {
    const reactions = { ...(m.reactions ?? {}) };
    if (emoji) reactions[reactor] = emoji;
    else delete reactions[reactor];
    return { ...m, reactions };
  });
}

/** Upgrade delivery status for the given message ids (never downgrades). */
export function applyReceipt(jid: string, ids: string[], status: MessageStatus) {
  const want = STATUS_RANK[status];
  const upgrade = (m: UiMessage): UiMessage | null =>
    want > STATUS_RANK[m.status ?? "sent"] ? { ...m, status } : null;

  const arr = messagesByChat.get(jid);
  const missing = new Set(ids);
  let changed = false;
  if (arr) {
    for (let i = 0; i < arr.length; i++) {
      if (!missing.has(arr[i].id)) continue;
      missing.delete(arr[i].id);
      const next = upgrade(arr[i]);
      if (next) {
        arr[i] = next;
        persistMessage(next);
        changed = true;
      }
    }
    if (changed) messagesByChat.set(jid, [...arr]);
  }
  // Receipts for messages no longer resident still update the stored copy.
  for (const id of missing) void patchPersistedMessage(jid, id, upgrade);
}
