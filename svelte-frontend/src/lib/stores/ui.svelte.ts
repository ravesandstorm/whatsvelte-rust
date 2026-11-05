// App-level transient UI state (not persisted).

import type { MediaDto } from "../types";
import type { UiMessage } from "./messages.svelte";

/** One item of a lightbox gallery (an album being browsed). */
export interface GalleryItem {
  media: MediaDto;
  thumbnail: string | null;
}

export const ui = $state({
  settingsOpen: false,
  /** When set, the full-screen media viewer (lightbox) is open for this item. */
  lightboxMedia: null as MediaDto | null,
  /** Inline base64 JPEG thumbnail for the open item, shown as an instant blurred
   * placeholder while the full-resolution media downloads. */
  lightboxThumbnail: null as string | null,
  /** When the lightbox was opened from an album: all its items (prev/next). */
  lightboxSet: null as GalleryItem[] | null,
  lightboxIndex: 0,
  /** When set, the album gallery modal is open for these messages. */
  albumItems: null as UiMessage[] | null,
});

/** Open the viewer on one item, optionally within a gallery for prev/next. */
export function openLightbox(
  media: MediaDto,
  thumbnail: string | null,
  set: GalleryItem[] | null = null,
  index = 0,
) {
  ui.lightboxSet = set;
  ui.lightboxIndex = index;
  ui.lightboxThumbnail = thumbnail;
  ui.lightboxMedia = media;
}

/** Step through the open gallery (no-op for a single item). */
export function stepLightbox(delta: number) {
  const set = ui.lightboxSet;
  if (!set || set.length < 2) return;
  const i = ui.lightboxIndex + delta;
  if (i < 0 || i >= set.length) return;
  ui.lightboxIndex = i;
  ui.lightboxThumbnail = set[i].thumbnail;
  ui.lightboxMedia = set[i].media;
}

/** Gallery items for an album's messages. */
export function galleryOf(items: UiMessage[]): GalleryItem[] {
  return items
    .filter((m) => !!m.media)
    .map((m) => ({ media: m.media!, thumbnail: m.thumbnail }));
}
