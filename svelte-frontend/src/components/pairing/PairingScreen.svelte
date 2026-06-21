<script lang="ts">
  import { session } from "../../lib/stores/session.svelte";
  import QrCanvas from "./QrCanvas.svelte";
  import PairCodeForm from "./PairCodeForm.svelte";
  import ConnectionStatus from "./ConnectionStatus.svelte";
</script>

<div class="screen">
  <div class="card">
    <header>
      <h1>WhatSvelte Rust</h1>
      <ConnectionStatus />
    </header>

    <h2>Link a device</h2>
    <ol class="steps">
      <li>Open WhatsApp on your phone</li>
      <li>Tap Menu → <b>Linked Devices</b></li>
      <li>Tap <b>Link a device</b> and scan this code</li>
    </ol>

    <div class="qr">
      {#if session.qrCode}
        <QrCanvas code={session.qrCode} />
      {:else}
        <div class="qr-wait">
          <div class="spinner"></div>
          <span>Waiting for QR…</span>
        </div>
      {/if}
    </div>

    <PairCodeForm />
  </div>
</div>

<style>
  .screen {
    height: 100%;
    display: grid;
    place-items: center;
    background: var(--wa-bg);
  }
  .card {
    width: 420px;
    max-width: 92vw;
    background: var(--wa-panel);
    border-radius: 14px;
    padding: 26px 30px 30px;
    display: flex;
    flex-direction: column;
    align-items: center;
    box-shadow: 0 10px 40px #0006;
  }
  header {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }
  h1 {
    font-size: 18px;
    margin: 0;
    color: var(--wa-green);
  }
  h2 {
    font-weight: 500;
    margin: 12px 0 6px;
  }
  .steps {
    color: var(--wa-text-muted);
    font-size: 13px;
    line-height: 1.7;
    margin: 0 0 16px;
    padding-left: 18px;
    align-self: flex-start;
  }
  .qr {
    min-height: 280px;
    display: grid;
    place-items: center;
    margin-bottom: 16px;
  }
  .qr-wait {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    color: var(--wa-text-muted);
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
</style>
