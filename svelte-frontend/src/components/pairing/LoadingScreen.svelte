<script lang="ts">
  // Shown on relaunch when the device is already paired (session.registered) but
  // the connection hasn't reported <success> yet — avoids flashing the QR.
  import ConnectionStatus from "./ConnectionStatus.svelte";
</script>

<div class="loading">
  <div class="spinner"></div>
  <p class="title">Loading your chats…</p>
  <ConnectionStatus />
</div>

<style>
  .loading {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    background: var(--wa-bg);
  }
  .spinner {
    width: 38px;
    height: 38px;
    border: 3px solid var(--wa-border);
    border-top-color: var(--wa-green);
    border-radius: 50%;
    animation: spin 0.9s linear infinite;
  }
  .title {
    font-size: 15px;
    color: var(--wa-text);
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
