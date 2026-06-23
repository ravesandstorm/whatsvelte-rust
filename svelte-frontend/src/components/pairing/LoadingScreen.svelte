<script lang="ts">
  // Shown on relaunch when the device is already paired (session.registered) but
  // the connection hasn't reported <success> yet — avoids flashing the QR.
  import ConnectionStatus from "./ConnectionStatus.svelte";
  import { resetAll } from "../../lib/stores/events";
  import { session } from "../../lib/stores/session.svelte";

  let resetting = $state(false);
  async function reset() {
    resetting = true;
    await resetAll();
    resetting = false;
  }

  // Drive a determinate bar off the offline-sync backlog when we have a total;
  // otherwise the spinner alone conveys an indeterminate wait.
  const total = $derived(session.syncTotalMessages);
  const pct = $derived(
    total > 0 ? Math.min(100, Math.round((session.syncDoneMessages / total) * 100)) : 0,
  );
</script>

<div class="loading">
  {#if total > 0}
    <p class="title">Loading your chats…</p>
    <div class="bar" role="progressbar" aria-valuenow={pct} aria-valuemin={0} aria-valuemax={100}>
      <div class="fill" style:width={`${pct}%`}></div>
    </div>
    <p class="count">{session.syncDoneMessages} of {total} messages · {pct}%</p>
  {:else}
    <div class="spinner"></div>
    <p class="title">Loading your chats…</p>
  {/if}
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
  .bar {
    width: 240px;
    max-width: 70vw;
    height: 6px;
    border-radius: 3px;
    background: var(--wa-panel-2);
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: var(--wa-green);
    border-radius: 3px;
    transition: width 0.25s ease;
  }
  .count {
    font-size: 12px;
    color: var(--wa-text-muted);
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
