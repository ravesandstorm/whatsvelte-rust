// Chat list state, reconstructed from wa://history (bootstrap) + wa://message.

import { SvelteMap } from "svelte/reactivity";
import type { ChatDto } from "../types";

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
  chats.set(d.jid, {
    jid: d.jid,
    name: d.name ?? prev?.name ?? null,
    lastMessage: d.lastMessage ?? prev?.lastMessage ?? null,
    timestamp: Math.max(d.timestamp, prev?.timestamp ?? 0),
    unread: d.unread || prev?.unread || 0,
  });
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
  chats.set(jid, {
    jid,
    name: prev?.name ?? null,
    lastMessage: lastMessage ?? prev?.lastMessage ?? null,
    timestamp: Math.max(timestamp, prev?.timestamp ?? 0),
    unread: incoming && !isActive ? (prev?.unread ?? 0) + 1 : (prev?.unread ?? 0),
  });
}

export function selectChat(jid: string) {
  chatUi.activeJid = jid;
  const c = chats.get(jid);
  if (c && c.unread) chats.set(jid, { ...c, unread: 0 });
}

export function setChatName(jid: string, name: string) {
  const c = chats.get(jid);
  if (c && !c.name) chats.set(jid, { ...c, name });
}

/** Reactive when read in a component template (reads the SvelteMap). */
export function sortedChats(): Chat[] {
  return [...chats.values()].sort((a, b) => b.timestamp - a.timestamp);
}
