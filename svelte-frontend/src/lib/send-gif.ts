// Send a remote GIF as an animated message. WhatsApp sends GIFs as looping
// videos (the backend sets `gif_playback` when the mimetype is image/gif), so we
// download the GIF bytes, stage them to a temp file, and reuse `send_media` with
// media type "video".
//
// The webview has no CSP (csp: null), so the GIF bytes can be fetched directly;
// once in a Blob they are same-origin, so the first-frame thumbnail can be drawn
// without tainting the canvas.

import { api } from "./ipc";
import { addOptimisticMedia, confirmOptimistic } from "./stores/messages.svelte";
import { touchChat } from "./stores/chats.svelte";
import { blobToTempFile, cleanupTemp, makeImageThumbnail } from "./send-media";

export async function sendGif(jid: string, gifUrl: string): Promise<void> {
  let blob: Blob;
  try {
    const res = await fetch(gifUrl);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    blob = await res.blob();
  } catch (e) {
    console.error("gif download failed", e);
    return;
  }

  // First-frame poster for the inline bubble (best-effort).
  let thumb: string | null = null;
  try {
    thumb = await makeImageThumbnail(blob);
  } catch {
    /* non-fatal — send without an inline thumbnail */
  }

  const now = Math.floor(Date.now() / 1000);
  const tempId = addOptimisticMedia(jid, "video", null, thumb);
  touchChat(jid, "🎞️ GIF", now, false);

  let tempPath: string | null = null;
  try {
    tempPath = await blobToTempFile(blob, "gif");
    const r = await api.sendMedia(jid, tempPath, "video", {
      caption: null,
      mimetype: "image/gif",
      jpegThumbnail: thumb,
    });
    confirmOptimistic(jid, tempId, r.messageId);
  } catch (e) {
    console.error("send gif failed", e);
  } finally {
    if (tempPath) await cleanupTemp(tempPath);
  }
}
