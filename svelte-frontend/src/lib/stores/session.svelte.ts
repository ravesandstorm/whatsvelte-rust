// Connection / auth state. A single reactive object mutated from events.ts.

import { displayName, isSelf } from "../util/jid";

export const session = $state({
  loggedIn: false,
  connected: false,
  jid: null as string | null,
  /** Our own LID (the self chat may be keyed by it before LID→PN is learned). */
  lid: null as string | null,
  /** Persistent "already paired" flag — show a loading screen (not the QR) on
   * relaunch while the connection comes up. */
  registered: false,
  /** Our own profile name ("username"). */
  pushName: null as string | null,
  /** Latest QR ref string (raw, to render offline). */
  qrCode: null as string | null,
  /** Latest phone-pairing code, if requested. */
  pairCode: null as string | null,
  /** True once the backend run loop has shut down (all QR codes expired) while
   * still unpaired. The on-screen QR is stale and the app must be relaunched. */
  clientDead: false,

  // --- Frontend hydration progress (loading cached chats from IndexedDB) ----
  // The backend has no chat/message query API; on restart the UI is rebuilt
  // from the webview's IndexedDB cache. That read + store-fill is the real
  // post-handshake wait, so we drive the loading screen off it (not any backend
  // sync signal, which never touches IndexedDB).
  /** True while rehydrating the cache on boot. */
  hydrating: false,
  /** Current step label, e.g. "Reading saved chats…". */
  hydrateLabel: "",
  /** Determinate progress denominator (0 = indeterminate / unknown yet). */
  hydrateTotal: 0,
  /** Determinate progress numerator. */
  hydrateDone: 0,

  /** True while history-sync chunks (wa://history) are streaming in. There's no
   * count or end signal for these, so it's an indeterminate indicator cleared by
   * a debounce once the chunks stop. Drives a subtle spinner in the chat header. */
  historySyncing: false,
});

/** The "message yourself" chat (by phone number or LID). */
export function isOwnChat(jid: string): boolean {
  return isSelf(jid, session.jid) || isSelf(jid, session.lid);
}

/** Title for a chat row/header: "<Name> (You)" for the self chat. */
export function chatTitle(jid: string, name?: string | null): string {
  if (isOwnChat(jid)) return `${session.pushName || name || displayName(session.jid ?? jid)} (You)`;
  return displayName(jid, name);
}
