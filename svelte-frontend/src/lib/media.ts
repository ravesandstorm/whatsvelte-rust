// Media resolution. The backend downloads + decrypts to a content-addressed
// file in the app-data cache and returns its path; we turn that into an
// asset-protocol URL the webview can load natively (no bytes over IPC/JS).
//
// The on-disk cache is the source of truth — download_media returns instantly on
// a cache hit (and survives restarts). We additionally memoize the in-flight
// promise per content hash so a re-rendered bubble doesn't re-invoke.

import { convertFileSrc } from "@tauri-apps/api/core";
import { api } from "./ipc";
import type { MediaDto } from "./types";

const inflight = new Map<string, Promise<string>>();

/** Resolve a media payload to a renderable asset URL (downloads on first use). */
export function mediaSrc(media: MediaDto): Promise<string> {
  const key = media.descriptor.fileSha256;
  let p = inflight.get(key);
  if (!p) {
    p = api
      .downloadMedia(media.descriptor, media.mimetype)
      .then((path) => convertFileSrc(path));
    inflight.set(key, p);
    // On failure, drop the memo so a later retry can re-attempt.
    p.catch(() => inflight.delete(key));
  }
  return p;
}

/** Human-readable duration (seconds → m:ss) for audio/video. */
export function formatDuration(secs: number | null): string {
  if (!secs || secs < 0) return "";
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60);
  return `${m}:${s.toString().padStart(2, "0")}`;
}
