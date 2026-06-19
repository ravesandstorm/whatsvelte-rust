// Per-chat message lists. In-memory only (no restart persistence by design).

import { SvelteMap } from "svelte/reactivity";
import type { MessageDto } from "../types";

export interface UiMessage extends MessageDto {
  status?: "sending" | "sent";
}

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

export function addOptimistic(jid: string, text: string): string {
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
  }
}

export function messagesFor(jid: string): UiMessage[] {
  return messagesByChat.get(jid) ?? [];
}
