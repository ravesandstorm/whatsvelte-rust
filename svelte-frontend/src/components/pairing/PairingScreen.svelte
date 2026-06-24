<script lang="ts">
  import { session } from "../../lib/stores/session.svelte";
  import QrCanvas from "./QrCanvas.svelte";
  import ConnectionStatus from "./ConnectionStatus.svelte";

  const dead = $derived(session.clientDead);
</script>

<div class="screen">
  <div class="card">
    <!-- Left: instructions, like WhatsApp Web -->
    <section class="left" class:dead>
      <header>
        <h1>WhatSvelte Rust</h1>
        <ConnectionStatus />
      </header>

      <h2>Steps to log in</h2>
      <ol class="steps">
        <li>Open <b>WhatsApp</b> on your phone</li>
        <li>Tap <b>Menu</b> on Android, or <b>Settings</b> on iPhone</li>
        <li>Tap <b>Linked devices</b>, then <b>Link a device</b></li>
        <li>Point your phone at this screen to scan the QR code</li>
      </ol>

      {#if dead}
        <div class="restart" role="alert">
          <span class="restart-icon">⚠</span>
          <div>
            <strong>This QR code has expired.</strong>
            <p>The connection closed. Please <b>restart the application</b> to get a fresh QR code.</p>
          </div>
        </div>
      {/if}
    </section>

    <!-- Right: the QR code -->
    <section class="right">
      <div class="qr" class:dead>
        {#if session.qrCode}
          <QrCanvas code={session.qrCode} />
        {:else if dead}
          <div class="qr-placeholder"></div>
        {:else}
          <div class="qr-wait">
            <div class="spinner"></div>
            <span>Waiting for QR…</span>
          </div>
        {/if}

        {#if dead}
          <div class="qr-invalid">
            <span class="qr-invalid-icon">⟳</span>
            <span class="qr-invalid-text">Expired — restart to refresh</span>
          </div>
        {/if}
      </div>
    </section>
  </div>
</div>

<style>
  .screen {
    height: 100%;
    display: grid;
    place-items: center;
    background: var(--wa-bg);
    padding: 24px;
    box-sizing: border-box;
  }
  .card {
    width: 760px;
    max-width: 95vw;
    background: var(--wa-panel);
    border-radius: 14px;
    padding: 36px 44px;
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 48px;
    align-items: center;
    box-shadow: 0 10px 40px #0006;
  }
  .left {
    min-width: 0;
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 18px;
  }
  h1 {
    font-size: 18px;
    margin: 0;
    color: var(--wa-green);
  }
  h2 {
    font-weight: 500;
    font-size: 22px;
    margin: 0 0 18px;
  }
  .steps {
    color: var(--wa-text);
    font-size: 14px;
    line-height: 1.5;
    margin: 0 0 20px;
    padding-left: 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .steps b {
    color: var(--wa-green);
    font-weight: 600;
  }
  /* When the client is dead, the instructions shift toward red to signal the
     session is no longer live. */
  .left.dead h2,
  .left.dead .steps {
    color: #e0736f;
  }
  .left.dead .steps b {
    color: #e0736f;
  }
  .restart {
    display: flex;
    gap: 12px;
    align-items: flex-start;
    background: rgba(224, 72, 61, 0.12);
    border: 1px solid rgba(224, 72, 61, 0.4);
    border-radius: 10px;
    padding: 12px 14px;
    margin-bottom: 20px;
    color: #f0a8a4;
  }
  .restart strong {
    color: #f3b6b2;
  }
  .restart p {
    margin: 4px 0 0;
    font-size: 13px;
    line-height: 1.45;
  }
  .restart b {
    color: #fff;
  }
  .restart-icon {
    font-size: 18px;
    line-height: 1.3;
  }
  .right {
    display: grid;
    place-items: center;
  }
  .qr {
    position: relative;
    width: 280px;
    height: 280px;
    display: grid;
    place-items: center;
  }
  /* Visibly invalidate the stale QR: desaturate + dim + blur it. */
  .qr.dead :global(canvas),
  .qr.dead .qr-placeholder {
    filter: grayscale(1) blur(3px) brightness(0.6);
  }
  .qr-placeholder {
    width: 264px;
    height: 264px;
    border-radius: 8px;
    background: #fff;
  }
  .qr-wait {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    color: var(--wa-text-muted);
  }
  .qr-invalid {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    border-radius: 8px;
    background: rgba(17, 27, 33, 0.55);
    color: #fff;
    text-align: center;
    pointer-events: none;
  }
  .qr-invalid-icon {
    font-size: 38px;
    line-height: 1;
  }
  .qr-invalid-text {
    font-size: 13px;
    font-weight: 600;
    padding: 0 16px;
  }
  .spinner {
    width: 34px;
    height: 34px;
    border: 3px solid var(--wa-border);
    border-top-color: var(--wa-green);
    border-radius: 50%;
    animation: spin 0.9s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  /* Narrow windows: stack the two columns. */
  @media (max-width: 720px) {
    .card {
      grid-template-columns: 1fr;
      gap: 28px;
      justify-items: center;
      padding: 28px 24px;
    }
    .left {
      width: 100%;
    }
  }
</style>
