// Connection / auth state. A single reactive object mutated from events.ts.

export const session = $state({
  loggedIn: false,
  connected: false,
  jid: null as string | null,
  /** Latest QR ref string (raw, to render offline). */
  qrCode: null as string | null,
  /** Latest phone-pairing code, if requested. */
  pairCode: null as string | null,
});
