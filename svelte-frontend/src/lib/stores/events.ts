// Single hub that registers all wa:// listeners once and routes payloads into
// the reactive stores. Also rehydrates persisted chats/messages on boot so the
// UI isn't empty on a plain restart (when no HistorySync event fires).

import { api, on } from "../ipc";
import { normalizeJid } from "../util/jid";
import { session } from "./session.svelte";
import {
  addHistoryMessages,
  addMessage,
  applyEdit,
  applyReaction,
  applyReceipt,
  applyRevoke,
  mergeMessages,
  messagesByChat,
  type MessageStatus,
} from "./messages.svelte";
import {
  applyChatFlags,
  chatUi,
  chats,
  ensureChat,
  mergeChats,
  setChatName,
  touchChat,
  upsertChatFromDto,
} from "./chats.svelte";
import {
  clearAll,
  deletePersisted,
  enablePersistence,
  loadSnapshot,
  schedulePersistChats,
  schedulePersistMessages,
  setAccount,
} from "../persist";
import type {
  ChatFlagsDto,
  HistoryDto,
  MessageDto,
  MessageUpdateDto,
  ReceiptDto,
} from "../types";

let started = false;

export async function startEventBridge() {
  if (started) return;
  started = true;

  // 1. Register listeners first so no live event is dropped while we hydrate.
  //    (Persistence is still disabled here, so store mutations don't write yet.)
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
    if (session.jid) await setAccount(session.jid);
  });
  await on("wa://auth/logged-out", async () => {
    session.loggedIn = false;
    session.connected = false;
    session.jid = null;
    // Drop the local cache so a different account can't see stale chats.
    chats.clear();
    messagesByChat.clear();
    chatUi.activeJid = null;
    await clearAll();
  });
  await on("wa://conn/state", () => {
    void refreshStatus();
  });

  await on<MessageDto>("wa://message", (m) => {
    addMessage(m);
    touchChat(m.chatJid, m.text, m.timestamp, !m.fromMe);
    if (m.pushName && !m.fromMe) setChatName(m.chatJid, m.pushName);
    void unifyLid(m.chatJid);
  });

  await on<HistoryDto>("wa://history", (h) => {
    for (const c of h.chats) {
      upsertChatFromDto(c);
      void unifyLid(c.jid);
    }
    addHistoryMessages(h.messages);
  });

  await on<ReceiptDto>("wa://receipt", (r) => {
    applyReceipt(r.chatJid, r.messageIds, r.status as MessageStatus);
  });

  await on<ChatFlagsDto>("wa://chat/flags", (f) => {
    applyChatFlags(f.jid, { muted: f.muted, pinned: f.pinned, archived: f.archived });
  });

  await on<MessageUpdateDto>("wa://message/update", (u) => {
    if (u.kind === "revoke") {
      applyRevoke(u.chatJid, u.targetId, {
        senderJid: u.senderJid ?? "",
        fromMe: u.fromMe,
        timestamp: u.timestamp,
      });
    } else if (u.kind === "edit") {
      applyEdit(u.chatJid, u.targetId, u.text, u.timestamp);
    } else if (u.kind === "reaction") {
      applyReaction(u.chatJid, u.targetId, normalizeJid(u.senderJid ?? ""), u.text ?? "");
    }
  });

  // 2. Find out who we are. NOTE: on a plain restart the backend boots/loads the
  //    session asynchronously, so auth_status can momentarily report
  //    loggedIn:false; a Connected event corrects it shortly after. Hydration
  //    below must NOT depend on this transient flag.
  await refreshStatus();

  // 3. Rehydrate the previous run's chats/messages from IndexedDB. This is the
  //    fix for "empty UI on restart": the backend only emits HistorySync once
  //    (at pairing), so a normal relaunch has no events to rebuild from.
  try {
    const snap = await loadSnapshot();
    const differentAccount =
      !!snap.accountJid &&
      !!session.jid &&
      normalizeJid(snap.accountJid) !== normalizeJid(session.jid);

    if (differentAccount) {
      // A different account is linked now — the cached chats aren't ours.
      await clearAll();
    } else {
      // Hydrate regardless of the (racy) loggedIn flag — cached data belongs to
      // this account. Merge semantics (upsert/addHistory dedupe) so anything a
      // live event added between step 1 and now isn't lost.
      for (const c of snap.chats) upsertChatFromDto(c);
      const all = Object.values(snap.messages).flat();
      if (all.length) addHistoryMessages(all);

      // Safety net: reconstruct any chat that has messages but no chat row, so
      // the list never depends on the chats store alone surviving.
      for (const [jid, items] of Object.entries(snap.messages)) {
        if (!items.length || chats.has(jid)) continue;
        const last = items[items.length - 1];
        ensureChat(jid, last.text ?? null, last.timestamp, last.pushName ?? null);
      }

      // We have cached chats → we were logged in. Show the chat view
      // immediately instead of flashing the pairing screen while the backend
      // finishes connecting (LoggedOut/Connected events still correct this).
      if (chats.size > 0) session.loggedIn = true;

      // Try to unify any cached LID conversations into their phone-number form.
      for (const jid of [...chats.keys()]) void unifyLid(jid);
    }
  } catch (e) {
    console.error("hydrate from cache failed", e);
  }

  // 4. From now on, mirror every store change to IndexedDB, and flush the
  //    current state once so a fresh pairing's history gets written immediately.
  if (session.jid) await setAccount(session.jid);
  enablePersistence();
  schedulePersistChats();
  for (const jid of messagesByChat.keys()) schedulePersistMessages(jid);
}

// JIDs we've already attempted to unify, so we don't re-query on every event.
const lidAttempted = new Set<string>();

/** If `jid` is a LID conversation with a known phone-number mapping, merge it
 * (chat row + messages, in memory and in IndexedDB) into the PN conversation. */
async function unifyLid(jid: string) {
  if (!jid.endsWith("@lid") || lidAttempted.has(jid)) return;
  lidAttempted.add(jid);
  try {
    const r = await api.resolveJid(jid);
    if (!r.pn || r.pn === jid) return;
    mergeMessages(jid, r.pn);
    mergeChats(jid, r.pn);
    await deletePersisted(jid);
    schedulePersistChats();
    schedulePersistMessages(r.pn);
  } catch (e) {
    console.error("lid unify failed", e);
  }
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
