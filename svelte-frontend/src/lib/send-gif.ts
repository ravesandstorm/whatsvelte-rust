// Send a picked GIF as an animated message. WhatsApp sends GIFs as looping MP4
// videos flagged `gifPlayback`; its clients can't play GIF bytes inside a video
// message. So we download KLIPY's MP4 rendition, stage it to a temp file and
// reuse `send_media` with media type "video". A result without an MP4
// rendition is sent as a still image (the GIF file itself) instead.
//
// Once fetched into a Blob the bytes are same-origin, so the poster frame can be
// drawn without tainting the canvas.

import { api } from "./ipc";
import type { Gif } from "./klipy";
import { addOptimisticMedia, confirmOptimistic, failOptimistic } from "./stores/messages.svelte";
import { touchChat } from "./stores/chats.svelte";
import { showToast } from "./stores/toast.svelte";
import {
  blobToTempFile,
  cleanupTemp,
  makeImageThumbnail,
  makeVideoThumbnail,
} from "./send-media";

export async function sendGif(jid: string, gif: Gif): Promise<void> {
  const asVideo = !!gif.mp4Url;
  let blob: Blob;
  try {
    const res = await fetch(gif.mp4Url ?? gif.gifUrl);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    blob = await res.blob();
  } catch (e) {
    console.error("gif download failed", e);
    showToast("Couldn't download the GIF");
    return;
  }

  // Poster frame for the inline bubble (best-effort).
  let thumb: string | null = null;
  let durationSecs: number | null = null;
  try {
    if (asVideo) {
      const meta = await makeVideoThumbnail(blob);
      thumb = meta.thumbnail;
      durationSecs = meta.durationSecs || null;
    } else {
      thumb = await makeImageThumbnail(blob);
    }
  } catch {
    /* non-fatal — send without an inline thumbnail */
  }

  const now = Math.floor(Date.now() / 1000);
  const tempId = addOptimisticMedia(jid, asVideo ? "video" : "image", null, thumb);
  touchChat(jid, "🎞️ GIF", now, false);

  let tempPath: string | null = null;
  try {
    tempPath = await blobToTempFile(blob, asVideo ? "mp4" : "gif");
    const r = asVideo
      ? await api.sendMedia(jid, tempPath, "video", {
          caption: null,
          mimetype: "video/mp4",
          jpegThumbnail: thumb,
          durationSecs,
          gifPlayback: true,
        })
      : await api.sendMedia(jid, tempPath, "image", {
          caption: null,
          mimetype: "image/gif",
          jpegThumbnail: thumb,
        });
    confirmOptimistic(jid, tempId, r.messageId);
  } catch (e) {
    console.error("send gif failed", e);
    failOptimistic(jid, tempId);
    showToast("GIF not sent");
  } finally {
    if (tempPath) await cleanupTemp(tempPath);
  }
}
