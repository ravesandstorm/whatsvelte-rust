<script lang="ts">
  // Shown on relaunch when the device is already paired (session.registered) but
  // the connection hasn't reported <success> yet — avoids flashing the QR.
  import ConnectionStatus from "./ConnectionStatus.svelte";
  import { resetAll } from "../../lib/stores/events";

  let resetting = $state(false);
  async function reset() {
    resetting = true;
    await resetAll();
    resetting = false;
  }
</script>

<div class="loading">
  <div class="spinner"></div>
  <p class="title">Loading your chats…</p>
  <ConnectionStatus />
  <button class="reset" onclick={reset} disabled={resetting}>
    {resetting ? "Resetting…" : "Stuck? Log out & reset"}
  </button>
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
  .reset {
    margin-top: 8px;
    border: 1px solid var(--wa-border);
    background: transparent;
    color: var(--wa-text-muted);
    font-size: 13px;
    padding: 6px 12px;
    border-radius: 6px;
  }
  .reset:hover:not(:disabled) {
    color: var(--wa-text);
    background: var(--wa-hover);
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
