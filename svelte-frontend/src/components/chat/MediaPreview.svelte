<script lang="ts">
  import { api } from "../../lib/ipc";
  import { addOptimisticMedia, confirmOptimistic } from "../../lib/stores/messages.svelte";
  import { touchChat } from "../../lib/stores/chats.svelte";
  import type { MediaSendOptions } from "../../lib/types";
  import {
    blobToTempFile,
    cleanupTemp,
    compressImage,
    extForMime,
    makeImageThumbnail,
    makeVideoThumbnail,
    type PendingAttachment,
  } from "../../lib/send-media";

  let {
    jid,
    attachment,
    onclose,
  }: { jid: string; attachment: PendingAttachment; onclose: () => void } = $props();

  let caption = $state("");
  let hd = $state(false);
  let sending = $state(false);

  const isImage = $derived(attachment.kind === "image");
  const title = $derived(
    attachment.kind === "image"
      ? "Send photo"
      : attachment.kind === "video"
        ? "Send video"
        : attachment.kind === "audio"
          ? "Send voice message"
          : "Send document",
  );

  async function send() {
    if (sending) return;
    sending = true;
    const now = Math.floor(Date.now() / 1000);

    let path: string;
    let tempPath: string | null = null;
    let thumb: string | null = null;
    let opts: MediaSendOptions;
    let label: string;

    try {
      if (attachment.kind === "image" && attachment.blob) {
        const out = await compressImage(attachment.blob, hd);
        thumb = await makeImageThumbnail(out);
        tempPath = await blobToTempFile(out, "jpg");
        path = tempPath;
        opts = { caption: caption || null, mimetype: "image/jpeg", jpegThumbnail: thumb };
        label = caption || "📷 Photo";
      } else if (attachment.kind === "video" && attachment.blob) {
        const meta = await makeVideoThumbnail(attachment.blob);
        thumb = meta.thumbnail;
        tempPath = await blobToTempFile(attachment.blob, extForMime(attachment.mimetype));
        path = tempPath;
        opts = {
          caption: caption || null,
          mimetype: attachment.mimetype ?? null,
          jpegThumbnail: thumb,
          durationSecs: attachment.durationSecs ?? meta.durationSecs,
        };
        label = caption || "🎬 Video";
      } else if (attachment.kind === "audio" && attachment.blob) {
        tempPath = await blobToTempFile(attachment.blob, extForMime(attachment.mimetype));
        path = tempPath;
        opts = {
          caption: caption || null,
          mimetype: attachment.mimetype ?? null,
          ptt: true,
          durationSecs: attachment.durationSecs ?? null,
        };
        label = "🎤 Voice message";
      } else {
        // Picked file sent as-is (backend reads its path) — documents.
        path = attachment.path ?? "";
        opts = {
          caption: caption || null,
          fileName: attachment.fileName ?? null,
          mimetype: attachment.mimetype ?? null,
        };
        label = caption || `📄 ${attachment.fileName ?? "Document"}`;
      }

      const optimisticText = thumb ? caption || null : label;
      const tempId = addOptimisticMedia(jid, attachment.kind, optimisticText, thumb);
      touchChat(jid, label, now, false);

      try {
        const r = await api.sendMedia(jid, path, attachment.kind, opts);
        confirmOptimistic(jid, tempId, r.messageId);
      } catch (e) {
        console.error("send media failed", e);
      }
    } finally {
      if (tempPath) await cleanupTemp(tempPath);
      sending = false;
      onclose();
    }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") onclose();
    else if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      void send();
    }
  }
</script>

<div
  class="overlay"
  role="presentation"
  onclick={(e) => {
    if (e.target === e.currentTarget) onclose();
  }}
>
  <div class="modal" role="dialog" aria-label="Send media">
    <header>
      <button class="close" aria-label="Cancel" onclick={onclose}>✕</button>
      <span class="title">{title}</span>
    </header>

    <div class="preview">
      {#if attachment.kind === "image" && attachment.previewUrl}
        <img src={attachment.previewUrl} alt="Selected" />
      {:else if attachment.kind === "video" && attachment.previewUrl}
        <!-- svelte-ignore a11y_media_has_caption -->
        <video src={attachment.previewUrl} controls></video>
      {:else if attachment.kind === "audio" && attachment.previewUrl}
        <audio src={attachment.previewUrl} controls></audio>
      {:else}
        <div class="doc">
          <div class="doc-icon">📄</div>
          <div class="doc-name">{attachment.fileName ?? "Document"}</div>
        </div>
      {/if}
    </div>

    <div class="controls">
      {#if isImage}
        <label class="hd">
          <input type="checkbox" bind:checked={hd} />
          <span>HD quality</span>
        </label>
      {/if}
      <!-- svelte-ignore a11y_autofocus -->
      <input
        class="caption"
        type="text"
        placeholder="Add a caption…"
        bind:value={caption}
        onkeydown={onKey}
        autofocus
      />
      <button class="send" onclick={send} disabled={sending} aria-label="Send">➤</button>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 2000;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.6);
  }
  .modal {
    width: min(560px, 92vw);
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    background: var(--wa-panel);
    border-radius: 10px;
    overflow: hidden;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.5);
  }
  header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    background: var(--wa-panel-2);
  }
  .close {
    border: none;
    background: transparent;
    color: var(--wa-text-muted);
    font-size: 16px;
  }
  .title {
    font-weight: 600;
  }
  .preview {
    flex: 1;
    min-height: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px;
    background: var(--wa-bg);
  }
  .preview img,
  .preview video {
    max-width: 100%;
    max-height: 56vh;
    border-radius: 6px;
    object-fit: contain;
  }
  .preview audio {
    width: 100%;
  }
  .doc {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 30px;
  }
  .doc-icon {
    font-size: 56px;
  }
  .doc-name {
    color: var(--wa-text);
    font-size: 14px;
    word-break: break-all;
    text-align: center;
  }
  .controls {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 16px;
    background: var(--wa-panel);
  }
  .hd {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    flex-shrink: 0;
    font-size: 13px;
    color: var(--wa-text-muted);
  }
  .caption {
    flex: 1;
    padding: 10px 14px;
    border: none;
    border-radius: 8px;
    background: var(--wa-panel-2);
    color: var(--wa-text);
  }
  .caption:focus {
    outline: none;
  }
  .send {
    width: 42px;
    height: 42px;
    border: none;
    border-radius: 50%;
    background: var(--wa-green);
    color: #04221c;
    font-size: 16px;
    flex-shrink: 0;
  }
  .send:disabled {
    opacity: 0.4;
  }
</style>
