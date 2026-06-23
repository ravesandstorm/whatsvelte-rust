// Per-chat message lists. In-memory only (no restart persistence by design).

import { SvelteMap } from "svelte/reactivity";
import type { MessageDto, QuotedDto } from "../types";
import { schedulePersistMessages } from "../persist";

export type MessageStatus = "sending" | "sent" | "delivered" | "read" | "played";

export interface UiMessage extends MessageDto {
  status?: MessageStatus;
  /** Set when the message was edited (epoch secs of the edit). */
  editedAt?: number;
  /** True when the message was deleted/revoked for everyone. */
  deleted?: boolean;
  /** Emoji reactions keyed by reactor JID ("" = none). */
  reactions?: Record<string, string>;
}

// Monotonic ordering so a later out-of-order receipt can't downgrade status.
const STATUS_RANK: Record<MessageStatus, number> = {
  sending: 0,
  sent: 1,
  delivered: 2,
  read: 3,
  played: 4,
};

export const messagesByChat = new SvelteMap<string, UiMessage[]>();

function ensure(jid: string): UiMessage[] {
  let arr = messagesByChat.get(jid);
  if (!arr) {
    arr = [];
    messagesByChat.set(jid, arr);
  }
  return arr;
}

function commit(jid: string, arr: UiMessage[]) {
  arr.sort((a, b) => a.timestamp - b.timestamp);
  // New array reference so SvelteMap consumers re-render.
  messagesByChat.set(jid, [...arr]);
  schedulePersistMessages(jid);
}

export function addMessage(m: UiMessage) {
  const arr = ensure(m.chatJid);
  if (m.id && arr.some((x) => x.id === m.id)) return; // dedupe by id

  // Reconcile the server echo of our own send with the optimistic bubble.
  if (m.fromMe) {
    const tmp = arr.findIndex((x) => x.status === "sending" && x.text === m.text);
    if (tmp >= 0) {
      arr[tmp] = { ...m, status: "sent" };
      commit(m.chatJid, arr);
      return;
    }
  }

  arr.push(m);
  commit(m.chatJid, arr);
}

export function addHistoryMessages(list: MessageDto[]) {
  const touched = new Set<string>();
  for (const m of list) {
    const arr = ensure(m.chatJid);
    if (m.id && arr.some((x) => x.id === m.id)) continue;
    arr.push(m);
    touched.add(m.chatJid);
  }
  for (const jid of touched) commit(jid, messagesByChat.get(jid)!);
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
    messagesByChat.set(jid, [...arr]);
    schedulePersistMessages(jid);
  }
}

export function messagesFor(jid: string): UiMessage[] {
  return messagesByChat.get(jid) ?? [];
}

/** Move messages from one chat key into another (LID→PN unification), deduping
 * by id. Removes the source key from memory (caller clears its persisted row). */
export function mergeMessages(fromJid: string, toJid: string) {
  const from = messagesByChat.get(fromJid);
  messagesByChat.delete(fromJid);
  if (!from || from.length === 0) return;
  const to = messagesByChat.get(toJid) ?? [];
  const seen = new Set(to.map((m) => m.id));
  for (const m of from) {
    if (m.id && seen.has(m.id)) continue;
    to.push({ ...m, chatJid: toJid });
    if (m.id) seen.add(m.id);
  }
  commit(toJid, to);
}

/** Mark a message deleted-for-everyone, inserting a tombstone if unseen. */
export function applyRevoke(
  jid: string,
  targetId: string,
  fallback?: { senderJid: string; fromMe: boolean; timestamp: number },
) {
  const arr = ensure(jid);
  const idx = arr.findIndex((m) => m.id === targetId);
  if (idx >= 0) {
    arr[idx] = { ...arr[idx], deleted: true, text: null, thumbnail: null, reactions: undefined };
  } else if (fallback) {
    arr.push({
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
    });
  } else {
    return;
  }
  commit(jid, arr);
}

/** Replace a message's text with its edited content. */
export function applyEdit(jid: string, targetId: string, text: string | null, editedAt: number) {
  const arr = messagesByChat.get(jid);
  if (!arr) return;
  const idx = arr.findIndex((m) => m.id === targetId);
  if (idx < 0) return;
  arr[idx] = { ...arr[idx], text, editedAt };
  messagesByChat.set(jid, [...arr]);
  schedulePersistMessages(jid);
}

/** Add/remove a reaction (empty emoji removes the reactor's reaction). */
export function applyReaction(
  jid: string,
  targetId: string,
  reactor: string,
  emoji: string,
) {
  const arr = messagesByChat.get(jid);
  if (!arr) return;
  const idx = arr.findIndex((m) => m.id === targetId);
  if (idx < 0) return;
  const reactions = { ...(arr[idx].reactions ?? {}) };
  if (emoji) reactions[reactor] = emoji;
  else delete reactions[reactor];
  arr[idx] = { ...arr[idx], reactions };
  messagesByChat.set(jid, [...arr]);
  schedulePersistMessages(jid);
}

/** Upgrade delivery status for the given message ids (never downgrades). */
export function applyReceipt(jid: string, ids: string[], status: MessageStatus) {
  const arr = messagesByChat.get(jid);
  if (!arr) return;
  const want = STATUS_RANK[status];
  let changed = false;
  const idSet = new Set(ids);
  for (let i = 0; i < arr.length; i++) {
    const m = arr[i];
    if (!idSet.has(m.id)) continue;
    const have = STATUS_RANK[m.status ?? "sent"];
    if (want > have) {
      arr[i] = { ...m, status };
      changed = true;
    }
  }
  if (changed) {
    messagesByChat.set(jid, [...arr]);
    schedulePersistMessages(jid);
  }
}
