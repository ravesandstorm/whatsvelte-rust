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
  hydrateChat,
  setChatName,
  touchChat,
  upsertChatFromDto,
} from "./chats.svelte";
import { canonicalJid, clearLidMap, loadLidMap, unifyLid } from "./lid";
import { clearNameCache, learnName, loadNameCache } from "./contacts.svelte";
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
/** Channels are fetched once per connected session (not in history sync). */
let newslettersLoaded = false;

/** History-sync chunks have no count/end signal, so we light an indeterminate
 * indicator on each chunk and clear it after a quiet gap. */
let historySyncTimer: ReturnType<typeof setTimeout> | null = null;
function markHistorySyncing() {
  session.historySyncing = true;
  if (historySyncTimer) clearTimeout(historySyncTimer);
  historySyncTimer = setTimeout(() => {
    historySyncTimer = null;
    session.historySyncing = false;
  }, 2500);
}

/** Pull the followed channels and seed them into the chat list so the Channels
 * section isn't empty (newsletters never arrive via history sync, only live).
 * Best-effort: requires an active connection, so failures are swallowed and a
 * later reconnect retries. */
async function fetchNewsletters() {
  try {
    const rows = await api.listNewsletters();
    for (const c of rows) {
      upsertChatFromDto(c);
      if (c.name) learnName(c.jid, c.name);
    }
  } catch (e) {
    newslettersLoaded = false; // allow a retry on the next connect
    console.error("list newsletters failed", e);
  }
}

/** Fetch channels the first time the session reports connected. */
async function maybeFetchNewsletters() {
  if (newslettersLoaded || !session.connected) return;
  newslettersLoaded = true;
  await fetchNewsletters();
}

/** Yield to the browser so the loading screen repaints between hydration chunks.
 * Without a yield the whole IndexedDB fill runs in one synchronous burst and the
 * progress bar never gets a frame to render (the old bug: the bar was instant). */
const nextFrame = (): Promise<void> =>
  new Promise((r) => {
    if (typeof requestAnimationFrame === "function") requestAnimationFrame(() => r());
    else setTimeout(r, 0);
  });

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
    // Manual or server-initiated logout — do a full reset so no stale state
    // (cache or device key) can survive into a re-pair.
    await resetAll();
  });
  await on("wa://conn/state", () => {
    void (async () => {
      await refreshStatus();
      await maybeFetchNewsletters();
    })();
  });

  await on<MessageDto>("wa://message", (m) => {
    // Canonicalize the chat key AND the sender key (LID→PN) so a contact that
    // switched addressing form lands in the one existing conversation, and group
    // participants resolve to a real phone/name instead of a raw `@lid`.
    const jid = canonicalJid(m.chatJid);
    const senderJid = canonicalJid(m.senderJid);
    const cm =
      jid === m.chatJid && senderJid === m.senderJid ? m : { ...m, chatJid: jid, senderJid };
    addMessage(cm);
    touchChat(jid, cm.text, cm.timestamp, !cm.fromMe);
    if (cm.pushName && !cm.fromMe) {
      setChatName(jid, cm.pushName);
      // Learn the *sender's* name too (matters for group participants, where the
      // sender JID differs from the chat JID) and persist it across restarts.
      learnName(senderJid || jid, cm.pushName);
    }
    // Learn the mapping (and merge any pre-existing split) for next time — for
    // both the chat JID and the participant sender JID.
    void unifyLid(m.chatJid);
    if (m.senderJid && m.senderJid !== m.chatJid) void unifyLid(m.senderJid);
  });

  await on<HistoryDto>("wa://history", (h) => {
    markHistorySyncing();
    for (const c of h.chats) {
      const jid = canonicalJid(c.jid);
      upsertChatFromDto(jid === c.jid ? c : { ...c, jid });
      if (c.name) learnName(jid, c.name);
      void unifyLid(c.jid);
    }
    addHistoryMessages(
      h.messages.map((m) => {
        const jid = canonicalJid(m.chatJid);
        const senderJid = canonicalJid(m.senderJid);
        if (m.pushName && !m.fromMe) learnName(senderJid || jid, m.pushName);
        if (m.senderJid && m.senderJid !== m.chatJid) void unifyLid(m.senderJid);
        return jid === m.chatJid && senderJid === m.senderJid
          ? m
          : { ...m, chatJid: jid, senderJid };
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
    const senderJid = canonicalJid(u.senderJid ?? "");
    if (u.kind === "revoke") {
      applyRevoke(jid, u.targetId, {
        senderJid,
        fromMe: u.fromMe,
        timestamp: u.timestamp,
      });
    } else if (u.kind === "edit") {
      applyEdit(jid, u.targetId, u.text, u.timestamp);
    } else if (u.kind === "reaction") {
      applyReaction(jid, u.targetId, normalizeJid(senderJid), u.text ?? "");
    }
    void unifyLid(u.chatJid);
    if (u.senderJid && u.senderJid !== u.chatJid) void unifyLid(u.senderJid);
  });

  // 2. Determine login state. The ONLY source of truth is the handshake type
  //    (reported as `registered`): IK ⇒ logged in, XX ⇒ logged out. It's a local
  //    DB read, so it's known even offline — we never log out a registered user
  //    just because the connection hasn't come up yet.
  await refreshStatus();

  // 3. Reconcile IndexedDB against the handshake truth.
  try {
    if (!session.registered) {
      // XX = logged out. Any IndexedDB data here is stale (e.g. a prior install
      // or a dead on-disk key that blocks QR generation). Wipe it and reset the
      // device key so a clean QR can be generated.
      const snap = await loadSnapshot();
      const hasData =
        snap.chats.length > 0 || Object.keys(snap.messages).length > 0 || !!snap.accountJid;
      if (hasData) {
        clearLidMap();
        clearNameCache();
        await clearAll();
        try {
          await api.resetSession();
        } catch (e) {
          console.error("reset session failed", e);
        }
        await refreshStatus();
      }
      // else: a clean unpaired boot — the QR will arrive shortly.
    } else {
      // IK = logged in. Rehydrate the previous run's chats/messages so the UI
      // isn't empty on restart (the backend only emits HistorySync at pairing).
      // This read + store-fill is the real post-handshake wait, so drive the
      // loading screen off it.
      session.hydrating = true;
      session.hydrateLabel = "Reading saved chats…";
      session.hydrateTotal = 0;
      session.hydrateDone = 0;
      await nextFrame(); // let the loading screen paint before the blocking read
      loadLidMap(session.jid);
      loadNameCache(session.jid);
      const snap = await loadSnapshot();
      const differentAccount =
        !!snap.accountJid &&
        !!session.jid &&
        normalizeJid(snap.accountJid) !== normalizeJid(session.jid);

      if (differentAccount) {
        // A different account is linked now — the cached chats aren't ours.
        clearLidMap();
        clearNameCache();
        await clearAll();
        session.hydrating = false;
      } else {
        // Canonicalize on the way in so a persisted LID chat with a known
        // mapping rehydrates straight into its PN conversation. Use hydrateChat
        // (not upsertChatFromDto) so the persisted mute/pin/archive flags are
        // preserved — otherwise the Archived/etc. sections vanish on restart.
        for (const c of snap.chats) {
          const jid = canonicalJid(c.jid);
          if (jid !== c.jid) void deletePersisted(c.jid); // drop the stale LID row
          hydrateChat(jid === c.jid ? c : { ...c, jid });
        }
        const all = Object.values(snap.messages)
          .flat()
          .map((m) => {
            const jid = canonicalJid(m.chatJid);
            return jid === m.chatJid ? m : { ...m, chatJid: jid };
          });
        // Insert messages in chunks, yielding a frame between each so the
        // progress bar actually advances on screen (a single addHistoryMessages
        // call would block the main thread and finish before any repaint).
        session.hydrateLabel = "Restoring messages…";
        session.hydrateTotal = all.length;
        session.hydrateDone = 0;
        const CHUNK = 400;
        for (let i = 0; i < all.length; i += CHUNK) {
          addHistoryMessages(all.slice(i, i + CHUNK));
          session.hydrateDone = Math.min(all.length, i + CHUNK);
          await nextFrame();
        }

        // Safety net: reconstruct any chat that has messages but no chat row.
        for (const [rawJid, items] of Object.entries(snap.messages)) {
          const jid = canonicalJid(rawJid);
          if (!items.length || chats.has(jid)) continue;
          const last = items[items.length - 1];
          ensureChat(jid, last.text ?? null, last.timestamp, last.pushName ?? null);
        }

        // Try to unify any cached LID conversations into their phone-number form.
        for (const jid of [...chats.keys()]) void unifyLid(jid);
        session.hydrating = false;
      }
    }
  } catch (e) {
    session.hydrating = false;
    console.error("hydrate from cache failed", e);
  }

  // 4. From now on, mirror every store change to IndexedDB, and flush the
  //    current state once so a fresh pairing's history gets written immediately.
  if (session.jid) await setAccount(session.jid);
  enablePersistence();
  schedulePersistChats();
  for (const jid of messagesByChat.keys()) schedulePersistMessages(jid);

  // If we're already connected at boot, seed the Channels section now (a later
  // wa://conn/state would also trigger this, but at startup it may not fire).
  void maybeFetchNewsletters();
}

export async function refreshStatus() {
  try {
    const s = await api.authStatus();
    session.loggedIn = s.loggedIn;
    session.connected = s.connected;
    session.jid = s.jid;
    session.registered = s.registered;
    session.pushName = s.pushName;
  } catch (e) {
    console.error("auth_status failed", e);
  }
}

/** Full client-side reset: clear in-memory + persisted state and regenerate the
 * device key (so a clean QR can be generated). Used on logout and as the manual
 * recovery from a stuck/stale session. */
export async function resetAll() {
  session.loggedIn = false;
  session.connected = false;
  session.registered = false;
  session.jid = null;
  session.pushName = null;
  session.hydrating = false;
  session.hydrateTotal = 0;
  session.hydrateDone = 0;
  session.historySyncing = false;
  if (historySyncTimer) {
    clearTimeout(historySyncTimer);
    historySyncTimer = null;
  }
  newslettersLoaded = false;
  chats.clear();
  messagesByChat.clear();
  chatUi.activeJid = null;
  clearLidMap();
  clearNameCache();
  await clearAll();
  try {
    await api.resetSession();
  } catch (e) {
    console.error("reset session failed", e);
  }
}
