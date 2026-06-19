// UI zoom controlled by Ctrl/Cmd + "+" / "-" / "0" (reset).
//
// The factor is applied as CSS `zoom` on <html> (supported by both webview
// engines Tauri uses — WebView2/Chromium and WKWebKit) so the whole UI scales
// uniformly, and persisted in localStorage so it survives restarts. We handle
// the shortcuts ourselves (rather than relying on the webview's native zoom)
// so behaviour and persistence are identical on every platform.

const KEY = "ui-zoom";
const MIN = 0.5;
const MAX = 2.0;
const STEP = 0.1;
const DEFAULT = 1.0;

let factor = DEFAULT;

function clamp(z: number): number {
  return Math.min(MAX, Math.max(MIN, Math.round(z * 100) / 100));
}

function apply() {
  // `zoom` scales layout (not just visuals) and avoids the blur of transform.
  document.documentElement.style.setProperty("zoom", String(factor));
}

function save() {
  try {
    localStorage.setItem(KEY, String(factor));
  } catch {
    /* storage may be unavailable; zoom still applies for the session */
  }
}

function set(z: number) {
  factor = clamp(z);
  apply();
  save();
}

export function zoomIn() {
  set(factor + STEP);
}

export function zoomOut() {
  set(factor - STEP);
}

export function zoomReset() {
  set(DEFAULT);
}

/** Install the global shortcut listener and restore the saved zoom. */
export function initZoom() {
  const saved = (() => {
    try {
      return parseFloat(localStorage.getItem(KEY) ?? "");
    } catch {
      return NaN;
    }
  })();
  factor = Number.isFinite(saved) ? clamp(saved) : DEFAULT;
  apply();

  window.addEventListener(
    "keydown",
    (e) => {
      // Cmd on macOS, Ctrl elsewhere.
      const mod = e.metaKey || e.ctrlKey;
      if (!mod) return;

      // "+" is Shift+"=" on most layouts; accept both the bare and shifted keys.
      switch (e.key) {
        case "+":
        case "=":
          e.preventDefault();
          zoomIn();
          break;
        case "-":
        case "_":
          e.preventDefault();
          zoomOut();
          break;
        case "0":
          e.preventDefault();
          zoomReset();
          break;
      }
    },
    { passive: false },
  );
}
