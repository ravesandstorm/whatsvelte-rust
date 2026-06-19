// Local, device-scoped preferences (not synced to WhatsApp). Persisted to
// localStorage and applied reactively. Server-synced chat settings (mute/pin/
// archive) live on the Chat object instead — see chats.svelte.ts (M9).

const KEY = "ui-settings";

export interface Settings {
  /** Global conversation wallpaper: a CSS color, preset id, or data-URL. */
  wallpaper: string | null;
  /** Per-chat wallpaper overrides, keyed by chat jid. */
  chatWallpaper: Record<string, string>;
  /** Enter sends (Shift+Enter newline) vs Enter inserts newline. */
  enterToSend: boolean;
  /** Whether to emit read receipts for messages scrolled into view (M5). */
  sendReadReceipts: boolean;
}

const DEFAULTS: Settings = {
  wallpaper: null,
  chatWallpaper: {},
  enterToSend: true,
  sendReadReceipts: true,
};

function load(): Settings {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return { ...DEFAULTS };
    return { ...DEFAULTS, ...(JSON.parse(raw) as Partial<Settings>) };
  } catch {
    return { ...DEFAULTS };
  }
}

export const settings = $state<Settings>(load());

function persist() {
  try {
    localStorage.setItem(KEY, JSON.stringify(settings));
  } catch {
    /* storage unavailable; settings apply for the session only */
  }
}

export function setWallpaper(value: string | null) {
  settings.wallpaper = value;
  persist();
}

export function setChatWallpaper(jid: string, value: string | null) {
  if (value == null) delete settings.chatWallpaper[jid];
  else settings.chatWallpaper[jid] = value;
  persist();
}

export function setEnterToSend(v: boolean) {
  settings.enterToSend = v;
  persist();
}

export function setSendReadReceipts(v: boolean) {
  settings.sendReadReceipts = v;
  persist();
}

/** Resolve the wallpaper to apply for a chat (per-chat override → global). */
export function wallpaperFor(jid: string): string | null {
  return settings.chatWallpaper[jid] ?? settings.wallpaper;
}

/** Turn a wallpaper value into a CSS background value. */
export function wallpaperCss(value: string | null): string {
  if (!value) return "var(--wa-chat-bg)";
  if (value.startsWith("data:") || value.startsWith("http") || value.startsWith("asset"))
    return `center / cover no-repeat url("${value}")`;
  // Otherwise treat it as a raw CSS color/gradient.
  return value;
}
