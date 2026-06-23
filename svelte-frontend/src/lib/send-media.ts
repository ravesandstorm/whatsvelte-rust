// Outgoing-media helpers: client-side image compression + thumbnailing (canvas,
// no heavy libs) and staging in-memory blobs to a temp file so the backend's
// `send_media` can read them by path (keeping large bytes off the IPC bridge).

import { join, tempDir } from "@tauri-apps/api/path";
import { remove, writeFile } from "@tauri-apps/plugin-fs";

export type SendKind = "image" | "video" | "audio" | "document";

/** A file the user picked or pasted, queued for the pre-send preview. */
export interface PendingAttachment {
  kind: SendKind;
  /** In-memory bytes (clipboard paste, picked image read for compression). */
  blob?: Blob;
  /** On-disk path (picked document/video sent as-is — backend reads it). */
  path?: string;
  fileName?: string;
  mimetype?: string;
  /** Audio/video length in seconds (recorder-measured). */
  durationSecs?: number;
  /** Object URL for previewing `blob` in the modal (caller revokes it). */
  previewUrl?: string;
}

/** File extension for a recorded/blob mimetype, for the staged temp file. */
export function extForMime(mime: string | undefined): string {
  if (!mime) return "bin";
  const m = mime.split(";")[0].trim();
  const map: Record<string, string> = {
    "image/jpeg": "jpg",
    "image/png": "png",
    "image/webp": "webp",
    "video/mp4": "mp4",
    "video/webm": "webm",
    "video/quicktime": "mov",
    "audio/mp4": "m4a",
    "audio/aac": "aac",
    "audio/webm": "webm",
    "audio/ogg": "ogg",
    "audio/mpeg": "mp3",
  };
  return map[m] ?? m.split("/")[1] ?? "bin";
}

const IMAGE_EXTS = ["png", "jpg", "jpeg", "gif", "webp", "bmp", "heic", "heif"];

function extOf(name: string): string {
  const i = name.lastIndexOf(".");
  return i >= 0 ? name.slice(i + 1).toLowerCase() : "";
}

export function isImageFile(name: string): boolean {
  return IMAGE_EXTS.includes(extOf(name));
}

/** Best-effort mimetype for a picked path (the backend defaults if unknown). */
export function mimeForPath(name: string): string | null {
  const e = extOf(name);
  const map: Record<string, string> = {
    pdf: "application/pdf",
    doc: "application/msword",
    docx: "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    xls: "application/vnd.ms-excel",
    xlsx: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    ppt: "application/vnd.ms-powerpoint",
    pptx: "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    txt: "text/plain",
    csv: "text/csv",
    zip: "application/zip",
    mp4: "video/mp4",
    mov: "video/quicktime",
    mp3: "audio/mpeg",
    ogg: "audio/ogg",
    m4a: "audio/mp4",
  };
  return map[e] ?? null;
}

async function loadBitmap(blob: Blob): Promise<ImageBitmap> {
  return await createImageBitmap(blob);
}

function drawScaled(bitmap: ImageBitmap, maxEdge: number): HTMLCanvasElement {
  const scale = Math.min(1, maxEdge / Math.max(bitmap.width, bitmap.height));
  const w = Math.max(1, Math.round(bitmap.width * scale));
  const h = Math.max(1, Math.round(bitmap.height * scale));
  const canvas = document.createElement("canvas");
  canvas.width = w;
  canvas.height = h;
  canvas.getContext("2d")!.drawImage(bitmap, 0, 0, w, h);
  return canvas;
}

function canvasToBlob(canvas: HTMLCanvasElement, quality: number): Promise<Blob> {
  return new Promise((resolve, reject) =>
    canvas.toBlob(
      (b) => (b ? resolve(b) : reject(new Error("canvas.toBlob returned null"))),
      "image/jpeg",
      quality,
    ),
  );
}

/** Re-encode an image to WhatsApp-ish standard (≤1600px, q0.75) or HD (≤3000px,
 * q0.92). Always JPEG. */
export async function compressImage(blob: Blob, hd: boolean): Promise<Blob> {
  const bitmap = await loadBitmap(blob);
  try {
    const canvas = drawScaled(bitmap, hd ? 3000 : 1600);
    return await canvasToBlob(canvas, hd ? 0.92 : 0.75);
  } finally {
    bitmap.close?.();
  }
}

/** Small base64 JPEG (no data: prefix) for the inline `jpegThumbnail`. */
export async function makeImageThumbnail(blob: Blob): Promise<string> {
  const bitmap = await loadBitmap(blob);
  try {
    const canvas = drawScaled(bitmap, 200);
    const dataUrl = canvas.toDataURL("image/jpeg", 0.6);
    return dataUrl.slice(dataUrl.indexOf(",") + 1);
  } finally {
    bitmap.close?.();
  }
}

/** Poster frame + intrinsic dimensions/duration for a recorded/picked video. */
export async function makeVideoThumbnail(
  blob: Blob,
): Promise<{ thumbnail: string | null; durationSecs: number; width: number; height: number }> {
  const url = URL.createObjectURL(blob);
  const video = document.createElement("video");
  video.muted = true;
  video.src = url;
  try {
    await new Promise<void>((resolve, reject) => {
      video.onloadeddata = () => resolve();
      video.onerror = () => reject(new Error("video decode failed"));
    });
    // Nudge past frame 0 so the poster isn't a black frame.
    await new Promise<void>((resolve) => {
      video.onseeked = () => resolve();
      video.currentTime = Math.min(0.1, video.duration || 0.1);
    });
    const canvas = document.createElement("canvas");
    const scale = Math.min(1, 320 / Math.max(video.videoWidth, video.videoHeight || 1));
    canvas.width = Math.max(1, Math.round(video.videoWidth * scale));
    canvas.height = Math.max(1, Math.round(video.videoHeight * scale));
    canvas.getContext("2d")!.drawImage(video, 0, 0, canvas.width, canvas.height);
    const dataUrl = canvas.toDataURL("image/jpeg", 0.6);
    return {
      thumbnail: dataUrl.slice(dataUrl.indexOf(",") + 1),
      durationSecs: Math.round(video.duration) || 0,
      width: video.videoWidth,
      height: video.videoHeight,
    };
  } catch {
    return { thumbnail: null, durationSecs: 0, width: 0, height: 0 };
  } finally {
    URL.revokeObjectURL(url);
  }
}

/** Write an in-memory blob to a temp file and return its absolute path. */
export async function blobToTempFile(blob: Blob, ext: string): Promise<string> {
  const dir = await tempDir();
  const name = `wa-send-${Date.now()}-${Math.random().toString(36).slice(2)}.${ext}`;
  const path = await join(dir, name);
  await writeFile(path, new Uint8Array(await blob.arrayBuffer()));
  return path;
}

/** Remove a staged temp file; best-effort (ignores errors). */
export async function cleanupTemp(path: string): Promise<void> {
  try {
    await remove(path);
  } catch {
    /* already gone / unwritable — nothing to do */
  }
}
