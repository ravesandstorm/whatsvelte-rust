// App theme (dark/light). Device-scoped, persisted to localStorage, and applied
// to the document root as `data-theme` so app.css can swap palettes. Default is
// "dark" (the original look). Mirrors the settings.svelte.ts persistence pattern.

export type Theme = "dark" | "light";

const KEY = "ui-theme";

function load(): Theme {
  try {
    const raw = localStorage.getItem(KEY);
    return raw === "light" || raw === "dark" ? raw : "dark";
  } catch {
    return "dark";
  }
}

export const themeState = $state<{ theme: Theme }>({ theme: load() });

function apply(theme: Theme) {
  try {
    document.documentElement.dataset.theme = theme;
  } catch {
    /* no document (tests); nothing to apply */
  }
}

export function setTheme(theme: Theme) {
  themeState.theme = theme;
  try {
    localStorage.setItem(KEY, theme);
  } catch {
    /* storage unavailable; theme applies for the session only */
  }
  apply(theme);
}

export function toggleTheme() {
  setTheme(themeState.theme === "dark" ? "light" : "dark");
}

/** Apply the persisted theme to the document root. Call once at startup. */
export function initTheme() {
  apply(themeState.theme);
}
