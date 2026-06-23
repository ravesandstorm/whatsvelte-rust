<script lang="ts">
  import { onDestroy } from "svelte";
  import type { PendingAttachment, SendKind } from "../../lib/send-media";

  let {
    mode,
    oncapture,
    onclose,
  }: {
    mode: "voice" | "video" | "photo";
    oncapture: (a: PendingAttachment) => void;
    onclose: () => void;
  } = $props();

  let stream: MediaStream | null = null;
  let recorder: MediaRecorder | null = null;
  let chunks: Blob[] = [];
  let videoEl: HTMLVideoElement | undefined = $state();

  let phase = $state<"init" | "ready" | "recording" | "error">("init");
  let errorMsg = $state("");
  let elapsed = $state(0);
  let startedAt = 0;
  let timer: ReturnType<typeof setInterval> | null = null;

  const needsVideo = $derived(mode !== "voice");
  const needsAudio = $derived(mode !== "photo");

  async function init() {
    try {
      stream = await navigator.mediaDevices.getUserMedia({
        video: needsVideo,
        audio: needsAudio,
      });
      phase = "ready";
      if (needsVideo && videoEl) {
        videoEl.srcObject = stream;
        await videoEl.play().catch(() => {});
      }
      // Voice/video start recording immediately; photo waits for a snapshot.
      if (mode !== "photo") startRecording();
    } catch (e) {
      phase = "error";
      errorMsg = e instanceof Error ? e.message : String(e);
    }
  }

  function startRecording() {
    if (!stream) return;
    chunks = [];
    recorder = new MediaRecorder(stream);
    recorder.ondataavailable = (e) => {
      if (e.data.size > 0) chunks.push(e.data);
    };
    recorder.onstop = finishRecording;
    recorder.start();
    phase = "recording";
    startedAt = Date.now();
    elapsed = 0;
    timer = setInterval(() => (elapsed = Math.floor((Date.now() - startedAt) / 1000)), 250);
  }

  function stopRecording() {
    if (timer) {
      clearInterval(timer);
      timer = null;
    }
    recorder?.stop(); // → onstop → finishRecording
  }

  function finishRecording() {
    const mimetype = recorder?.mimeType || (mode === "video" ? "video/webm" : "audio/webm");
    const blob = new Blob(chunks, { type: mimetype });
    const kind: SendKind = mode === "video" ? "video" : "audio";
    emit({
      kind,
      blob,
      mimetype,
      durationSecs: Math.round((Date.now() - startedAt) / 1000),
      previewUrl: URL.createObjectURL(blob),
    });
  }

  function takePhoto() {
    if (!videoEl) return;
    const canvas = document.createElement("canvas");
    canvas.width = videoEl.videoWidth;
    canvas.height = videoEl.videoHeight;
    canvas.getContext("2d")!.drawImage(videoEl, 0, 0);
    canvas.toBlob(
      (blob) => {
        if (!blob) return;
        emit({
          kind: "image",
          blob,
          fileName: "photo.jpg",
          mimetype: "image/jpeg",
          previewUrl: URL.createObjectURL(blob),
        });
      },
      "image/jpeg",
      0.95,
    );
  }

  // Hand the capture upward, then tear down (oncapture opens the preview modal).
  function emit(att: PendingAttachment) {
    teardown();
    oncapture(att);
  }

  function teardown() {
    if (timer) {
      clearInterval(timer);
      timer = null;
    }
    if (recorder && recorder.state !== "inactive") {
      recorder.onstop = null;
      recorder.stop();
    }
    stream?.getTracks().forEach((t) => t.stop());
    stream = null;
  }

  function cancel() {
    teardown();
    onclose();
  }

  function fmt(s: number): string {
    const m = Math.floor(s / 60);
    const sec = s % 60;
    return `${m}:${sec.toString().padStart(2, "0")}`;
  }

  $effect(() => {
    void init();
  });
  onDestroy(teardown);
</script>

<div
  class="overlay"
  role="presentation"
  onclick={(e) => {
    if (e.target === e.currentTarget) cancel();
  }}
>
  <div class="modal" role="dialog" aria-label="Record media">
    <header>
      <button class="close" aria-label="Cancel" onclick={cancel}>✕</button>
      <span class="title">
        {mode === "photo" ? "Take photo" : mode === "video" ? "Record video" : "Record voice"}
      </span>
    </header>

    <div class="stage">
      {#if phase === "error"}
        <div class="err">
          <p>Couldn't access your {needsVideo ? "camera" : "microphone"}.</p>
          <p class="detail">{errorMsg}</p>
        </div>
      {:else if needsVideo}
        <!-- svelte-ignore a11y_media_has_caption -->
        <video bind:this={videoEl} muted playsinline></video>
      {:else}
        <div class="mic">🎤</div>
      {/if}

      {#if phase === "recording" || (mode === "voice" && phase === "ready")}
        <div class="timer"><span class="dot"></span>{fmt(elapsed)}</div>
      {/if}
    </div>

    <div class="controls">
      {#if phase === "error"}
        <button class="btn" onclick={cancel}>Close</button>
      {:else if mode === "photo"}
        <button class="btn primary" onclick={takePhoto} disabled={phase !== "ready"}>
          Capture
        </button>
      {:else}
        <button class="btn primary" onclick={stopRecording} disabled={phase !== "recording"}>
          Stop &amp; preview
        </button>
      {/if}
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
  .stage {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 220px;
    padding: 16px;
    background: var(--wa-bg);
  }
  .stage video {
    max-width: 100%;
    max-height: 56vh;
    border-radius: 6px;
    background: #000;
  }
  .mic {
    font-size: 72px;
    opacity: 0.8;
  }
  .timer {
    position: absolute;
    top: 24px;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 4px 12px;
    border-radius: 14px;
    background: rgba(0, 0, 0, 0.55);
    color: #fff;
    font-variant-numeric: tabular-nums;
  }
  .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: #e0483d;
    animation: blink 1s steps(2, start) infinite;
  }
  @keyframes blink {
    50% {
      opacity: 0.2;
    }
  }
  .err {
    text-align: center;
    color: var(--wa-text-muted);
  }
  .err .detail {
    font-size: 12px;
    opacity: 0.7;
  }
  .controls {
    display: flex;
    justify-content: center;
    padding: 14px;
    background: var(--wa-panel);
  }
  .btn {
    border: 1px solid var(--wa-border);
    background: var(--wa-panel-2);
    color: var(--wa-text);
    padding: 10px 20px;
    border-radius: 20px;
    font-size: 14px;
  }
  .btn.primary {
    background: var(--wa-green);
    border-color: var(--wa-green);
    color: #04221c;
  }
  .btn:disabled {
    opacity: 0.4;
  }
</style>
