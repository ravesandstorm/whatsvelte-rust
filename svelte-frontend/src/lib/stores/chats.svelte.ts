// Chat list state, reconstructed from wa://history (bootstrap) + wa://message.

import { SvelteMap } from "svelte/reactivity";
import type { ChatDto } from "../types";
import { schedulePersistChats } from "../persist";
import { isNewsletter, isStatus, isStatusBroadcast } from "../util/jid";

export interface Chat {
  jid: string;
  name: string | null;
  lastMessage: string | null;
  timestamp: number;
  unread: number;
  muted?: boolean;
  pinned?: boolean;
  archived?: boolean;
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
    // Preserve server-synced flags — a history/message upsert must never wipe
    // an existing pin/mute/archive state.
    muted: prev?.muted,
    pinned: prev?.pinned,
    archived: prev?.archived,
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
    // A new message must not unpin/unmute/unarchive the chat.
    muted: prev?.muted,
    pinned: prev?.pinned,
    archived: prev?.archived,
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

/** Apply a server-synced flag change (mute/pin/archive). */
export function applyChatFlags(
  jid: string,
  flags: { muted?: boolean | null; pinned?: boolean | null; archived?: boolean | null },
) {
  const prev = chats.get(jid);
  if (!prev) return; // flag for a chat we don't know about yet — ignore
  chats.set(jid, {
    ...prev,
    muted: flags.muted ?? prev.muted,
    pinned: flags.pinned ?? prev.pinned,
    archived: flags.archived ?? prev.archived,
  });
  schedulePersistChats();
}

/** Optimistically set a flag locally (before/independent of the server echo). */
export function setChatFlag(jid: string, flag: "muted" | "pinned" | "archived", value: boolean) {
  const prev = chats.get(jid);
  if (!prev) return;
  chats.set(jid, { ...prev, [flag]: value });
  schedulePersistChats();
}

/** Merge the `fromJid` chat row into `toJid` (LID→PN unification), preferring
 * the newer row's preview and summing unread. Removes the source row. */
export function mergeChats(fromJid: string, toJid: string) {
  const from = chats.get(fromJid);
  if (!from) return;
  const to = chats.get(toJid);
  if (to) {
    const newer = from.timestamp >= to.timestamp ? from : to;
    chats.set(toJid, {
      jid: toJid,
      name: to.name ?? from.name,
      lastMessage: newer.lastMessage,
      timestamp: Math.max(to.timestamp, from.timestamp),
      unread: (to.unread ?? 0) + (from.unread ?? 0),
      muted: to.muted ?? from.muted,
      pinned: to.pinned ?? from.pinned,
      archived: to.archived ?? from.archived,
    });
  } else {
    chats.set(toJid, { ...from, jid: toJid });
  }
  chats.delete(fromJid);
  if (chatUi.activeJid === fromJid) chatUi.activeJid = toJid;
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

/** A regular 1:1/group conversation — excludes channels (newsletters) and the
 * status feed, which live in their own sections. */
function isRegular(c: Chat): boolean {
  return !isNewsletter(c.jid) && !isStatusBroadcast(c.jid) && !isStatus(c.jid);
}

/** The active (non-archived) chat list. Pinned chats float to the top; everyone
 * else sorts by most-recent activity (WhatsApp behaviour — muting does NOT
 * reorder a chat). Channels and status updates are sectioned out. */
export function sortedChats(): Chat[] {
  return [...chats.values()]
    .filter((c) => !c.archived && isRegular(c))
    .sort((a, b) => {
      const pin = Number(!!b.pinned) - Number(!!a.pinned);
      if (pin !== 0) return pin;
      return b.timestamp - a.timestamp;
    });
}

/** Archived chats, most-recent first, for the dedicated Archived section. */
export function archivedChats(): Chat[] {
  return [...chats.values()]
    .filter((c) => c.archived)
    .sort((a, b) => b.timestamp - a.timestamp);
}

/** Channels / newsletters, most-recent first, for the Channels section. */
export function channelChats(): Chat[] {
  return [...chats.values()]
    .filter((c) => !c.archived && isNewsletter(c.jid))
    .sort((a, b) => b.timestamp - a.timestamp);
}

/** Status-update feeds, most-recent first, for the Status section. */
export function statusChats(): Chat[] {
  return [...chats.values()]
    .filter((c) => !c.archived && (isStatusBroadcast(c.jid) || isStatus(c.jid)))
    .sort((a, b) => b.timestamp - a.timestamp);
}
