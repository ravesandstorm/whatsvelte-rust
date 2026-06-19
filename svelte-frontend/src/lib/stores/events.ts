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
  messagesByChat,
  type MessageStatus,
} from "./messages.svelte";
import {
  applyChatFlags,
  chatUi,
  chats,
  ensureChat,
  setChatName,
  touchChat,
  upsertChatFromDto,
} from "./chats.svelte";
import { canonicalJid, clearLidMap, loadLidMap, unifyLid } from "./lid";
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
    clearLidMap();
    await clearAll();
  });
  await on("wa://conn/state", () => {
    void refreshStatus();
  });

  await on<MessageDto>("wa://message", (m) => {
    // Canonicalize the chat key (LID→PN) so a contact that switched addressing
    // form lands in the one existing conversation instead of a split twin.
    const jid = canonicalJid(m.chatJid);
    const cm = jid === m.chatJid ? m : { ...m, chatJid: jid };
    addMessage(cm);
    touchChat(jid, cm.text, cm.timestamp, !cm.fromMe);
    if (cm.pushName && !cm.fromMe) setChatName(jid, cm.pushName);
    // Learn the mapping (and merge any pre-existing split) for next time.
    void unifyLid(m.chatJid);
  });

  await on<HistoryDto>("wa://history", (h) => {
    for (const c of h.chats) {
      const jid = canonicalJid(c.jid);
      upsertChatFromDto(jid === c.jid ? c : { ...c, jid });
      void unifyLid(c.jid);
    }
    addHistoryMessages(
      h.messages.map((m) => {
        const jid = canonicalJid(m.chatJid);
        return jid === m.chatJid ? m : { ...m, chatJid: jid };
      }),
    );
  });

  await on<ReceiptDto>("wa://receipt", (r) => {
    applyReceipt(canonicalJid(r.chatJid), r.messageIds, r.status as MessageStatus);
    void unifyLid(r.chatJid);
  });

  await on<ChatFlagsDto>("wa://chat/flags", (f) => {
    applyChatFlags(canonicalJid(f.jid), {
      muted: f.muted,
      pinned: f.pinned,
      archived: f.archived,
    });
    void unifyLid(f.jid);
  });

  await on<MessageUpdateDto>("wa://message/update", (u) => {
    const jid = canonicalJid(u.chatJid);
    if (u.kind === "revoke") {
      applyRevoke(jid, u.targetId, {
        senderJid: u.senderJid ?? "",
        fromMe: u.fromMe,
        timestamp: u.timestamp,
      });
    } else if (u.kind === "edit") {
      applyEdit(jid, u.targetId, u.text, u.timestamp);
    } else if (u.kind === "reaction") {
      applyReaction(jid, u.targetId, normalizeJid(u.senderJid ?? ""), u.text ?? "");
    }
    void unifyLid(u.chatJid);
  });

  // 2. Find out who we are. NOTE: on a plain restart the backend boots/loads the
  //    session asynchronously, so auth_status can momentarily report
  //    loggedIn:false; a Connected event corrects it shortly after. Hydration
  //    below must NOT depend on this transient flag.
  await refreshStatus();

  // 2b. Load this account's learned LID→PN map before hydration so persisted
  //     LID chats canonicalize/merge correctly on boot.
  loadLidMap(session.jid);

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
      clearLidMap();
      await clearAll();
    } else {
      // Hydrate regardless of the (racy) loggedIn flag — cached data belongs to
      // this account. Merge semantics (upsert/addHistory dedupe) so anything a
      // live event added between step 1 and now isn't lost.
      // Canonicalize on the way in so a persisted LID chat with a known mapping
      // rehydrates straight into its PN conversation.
      for (const c of snap.chats) {
        const jid = canonicalJid(c.jid);
        if (jid !== c.jid) void deletePersisted(c.jid); // drop the stale LID row
        upsertChatFromDto(jid === c.jid ? c : { ...c, jid });
      }
      const all = Object.values(snap.messages)
        .flat()
        .map((m) => {
          const jid = canonicalJid(m.chatJid);
          return jid === m.chatJid ? m : { ...m, chatJid: jid };
        });
      if (all.length) addHistoryMessages(all);

      // Safety net: reconstruct any chat that has messages but no chat row, so
      // the list never depends on the chats store alone surviving.
      for (const [rawJid, items] of Object.entries(snap.messages)) {
        const jid = canonicalJid(rawJid);
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
