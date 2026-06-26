// KLIPY GIF API client (https://docs.klipy.com/gifs-api).
//
// The app key is read from the `VITE_KLIPY_API_KEY` build-time env var so it is
// never hardcoded. Set it in `svelte-frontend/.env` (or the shell) before
// building, e.g. `VITE_KLIPY_API_KEY=your_key_here`.
//
// The webview has no CSP (tauri.conf.json `csp: null`), so these requests and
// the returned GIF media URLs load directly from KLIPY's CDN.

const API_KEY: string = import.meta.env.VITE_KLIPY_API_KEY ?? "";
const BASE = "https://api.klipy.com/api/v1";
const PER_PAGE = 24;

/** Whether an API key is configured; the GIF tab shows a hint when it isn't. */
export const klipyConfigured = API_KEY.length > 0;

/** A GIF normalized down to just what the picker needs. */
export interface Gif {
  id: string;
  /** Small looping preview shown in the grid. */
  previewUrl: string;
  /** Full-size GIF used when sending. */
  gifUrl: string;
  width: number;
  height: number;
}

export interface GifPage {
  gifs: Gif[];
  hasNext: boolean;
}

// --- KLIPY response shapes (only the fields we read) ---
interface KlipyFormat {
  url?: string;
  width?: number;
  height?: number;
}
interface KlipyTier {
  gif?: KlipyFormat;
  webp?: KlipyFormat;
}
interface KlipyItem {
  id: number | string;
  file?: { hd?: KlipyTier; md?: KlipyTier; sm?: KlipyTier; xs?: KlipyTier };
}
interface KlipyResponse {
  result?: boolean;
  data?: { data?: KlipyItem[]; has_next?: boolean };
}

function normalize(item: KlipyItem): Gif | null {
  const f = item.file ?? {};
  // Prefer a lightweight webp/gif for the grid; fall back across tiers so a
  // missing size doesn't drop an otherwise-usable result.
  const preview = f.sm?.webp ?? f.sm?.gif ?? f.md?.webp ?? f.md?.gif ?? f.xs?.gif;
  const full = f.md?.gif ?? f.hd?.gif ?? f.sm?.gif ?? preview;
  if (!preview?.url || !full?.url) return null;
  return {
    id: String(item.id),
    previewUrl: preview.url,
    gifUrl: full.url,
    width: preview.width ?? 0,
    height: preview.height ?? 0,
  };
}

async function query(path: string, params: Record<string, string>): Promise<GifPage> {
  if (!klipyConfigured) throw new Error("VITE_KLIPY_API_KEY is not set");
  const qs = new URLSearchParams({ per_page: String(PER_PAGE), ...params });
  const res = await fetch(`${BASE}/${API_KEY}/gifs/${path}?${qs.toString()}`);
  if (!res.ok) throw new Error(`KLIPY request failed (${res.status})`);
  const json = (await res.json()) as KlipyResponse;
  const items = json.data?.data ?? [];
  return {
    gifs: items.map(normalize).filter((g): g is Gif => g !== null),
    hasNext: !!json.data?.has_next,
  };
}

/** Trending GIFs (the default view when the GIF tab opens). */
export function klipyTrending(page = 1): Promise<GifPage> {
  return query("trending", { page: String(page) });
}

/** Keyword search. */
export function klipySearch(q: string, page = 1): Promise<GifPage> {
  return query("search", { q, page: String(page) });
}
