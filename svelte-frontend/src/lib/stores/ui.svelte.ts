// App-level transient UI state (not persisted).

import type { MediaDto } from "../types";

export const ui = $state({
  settingsOpen: false,
  /** When set, the full-screen media viewer (lightbox) is open for this item. */
  lightboxMedia: null as MediaDto | null,
  /** Inline base64 JPEG thumbnail for the open item, shown as an instant blurred
   * placeholder while the full-resolution media downloads. */
  lightboxThumbnail: null as string | null,
});
