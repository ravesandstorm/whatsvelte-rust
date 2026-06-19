// Single hub that registers all wa:// listeners once and routes payloads into
// the reactive stores.

import { api, on } from "../ipc";
import { session } from "./session.svelte";
import { addHistoryMessages, addMessage } from "./messages.svelte";
import { setChatName, touchChat, upsertChatFromDto } from "./chats.svelte";
import type { HistoryDto, MessageDto } from "../types";

let started = false;

export async function startEventBridge() {
  if (started) return;
  started = true;

  await on<{ code: string }>("wa://auth/qr", (p) => {
    session.qrCode = p.code;
  });
  await on<{ code: string }>("wa://auth/pair-code", (p) => {
    session.pairCode = p.code;
  });
  await on("wa://auth/paired", async () => {
    session.qrCode = null;
    session.pairCode = null;
    await refreshStatus();
  });
  await on("wa://auth/logged-out", () => {
    session.loggedIn = false;
    session.connected = false;
    session.jid = null;
  });
  await on("wa://conn/state", () => {
    void refreshStatus();
  });

  await on<MessageDto>("wa://message", (m) => {
    addMessage(m);
    touchChat(m.chatJid, m.text, m.timestamp, !m.fromMe);
    if (m.pushName && !m.fromMe) setChatName(m.chatJid, m.pushName);
  });

  await on<HistoryDto>("wa://history", (h) => {
    for (const c of h.chats) upsertChatFromDto(c);
    addHistoryMessages(h.messages);
  });

  await refreshStatus();
}

export async function refreshStatus() {
  try {
    const s = await api.authStatus();
    session.loggedIn = s.loggedIn;
    session.connected = s.connected;
    session.jid = s.jid;
  } catch (e) {
    console.error("auth_status failed", e);
  }
}
