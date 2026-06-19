// Chat list state, reconstructed from wa://history (bootstrap) + wa://message.

import { SvelteMap } from "svelte/reactivity";
import type { ChatDto } from "../types";
import { schedulePersistChats } from "../persist";

export interface Chat {
  jid: string;
  name: string | null;
  lastMessage: string | null;
  timestamp: number;
  unread: number;
}

export const chats = new SvelteMap<string, Chat>();
export const chatUi = $state({ activeJid: null as string | null });

/** Upsert from a history-sync chat row (keeps the higher timestamp / existing name). */
export function upsertChatFromDto(d: ChatDto) {
  const prev = chats.get(d.jid);
  // Only let the newer row drive the preview, so the list never shows an older
  // message's text next to a newer message's time (chunks can arrive any order).
  const isNewer = !prev || d.timestamp >= prev.timestamp;
  chats.set(d.jid, {
    jid: d.jid,
    name: d.name ?? prev?.name ?? null,
    lastMessage: isNewer ? (d.lastMessage ?? prev?.lastMessage ?? null) : prev?.lastMessage ?? null,
    timestamp: Math.max(d.timestamp, prev?.timestamp ?? 0),
    unread: d.unread || prev?.unread || 0,
  });
  schedulePersistChats();
}

/** Bump a chat on a new message (creating it if unseen). */
export function touchChat(
  jid: string,
  lastMessage: string | null,
  timestamp: number,
  incoming: boolean,
) {
  const prev = chats.get(jid);
  const isActive = chatUi.activeJid === jid;
  const isNewer = !prev || timestamp >= prev.timestamp;
  chats.set(jid, {
    jid,
    name: prev?.name ?? null,
    lastMessage: isNewer ? (lastMessage ?? prev?.lastMessage ?? null) : prev?.lastMessage ?? null,
    timestamp: Math.max(timestamp, prev?.timestamp ?? 0),
    unread: incoming && !isActive ? (prev?.unread ?? 0) + 1 : (prev?.unread ?? 0),
  });
  schedulePersistChats();
}

/**
 * Create a chat entry if one doesn't exist yet. Used to reconstruct the chat
 * list from persisted messages when the chats store wasn't restored (so the
 * list never depends solely on the chats cache surviving).
 */
export function ensureChat(
  jid: string,
  lastMessage: string | null,
  timestamp: number,
  name: string | null,
) {
  if (chats.has(jid)) return;
  chats.set(jid, { jid, name, lastMessage, timestamp, unread: 0 });
  schedulePersistChats();
}

export function selectChat(jid: string) {
  chatUi.activeJid = jid;
  const c = chats.get(jid);
  if (c && c.unread) {
    chats.set(jid, { ...c, unread: 0 });
    schedulePersistChats();
  }
}

export function setChatName(jid: string, name: string) {
  const c = chats.get(jid);
  if (c && !c.name) {
    chats.set(jid, { ...c, name });
    schedulePersistChats();
  }
}

/** Reactive when read in a component template (reads the SvelteMap). */
export function sortedChats(): Chat[] {
  return [...chats.values()].sort((a, b) => b.timestamp - a.timestamp);
}
