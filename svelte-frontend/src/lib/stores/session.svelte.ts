// Connection / auth state. A single reactive object mutated from events.ts.

export const session = $state({
  loggedIn: false,
  connected: false,
  jid: null as string | null,
  /** Persistent "already paired" flag — show a loading screen (not the QR) on
   * relaunch while the connection comes up. */
  registered: false,
  /** Our own profile name ("username"). */
  pushName: null as string | null,
  /** Latest QR ref string (raw, to render offline). */
  qrCode: null as string | null,
  /** Latest phone-pairing code, if requested. */
  pairCode: null as string | null,

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
});
