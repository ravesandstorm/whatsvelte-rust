<script lang="ts">
  import { dismissToast, toasts } from "../../lib/stores/toast.svelte";
</script>

<div class="toasts" role="status" aria-live="polite">
  {#each toasts as t (t.id)}
    <button class="toast" onclick={() => dismissToast(t.id)}>{t.text}</button>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    bottom: 84px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 2000;
    display: flex;
    flex-direction: column;
    gap: 6px;
    align-items: center;
    pointer-events: none;
  }
  .toast {
    pointer-events: auto;
    border: 1px solid var(--wa-border);
    background: var(--wa-panel-2);
    color: var(--wa-text);
    font: inherit;
    font-size: 13.5px;
    padding: 8px 14px;
    border-radius: 8px;
    box-shadow: 0 6px 18px rgba(0, 0, 0, 0.35);
    cursor: pointer;
    max-width: min(420px, calc(100vw - 32px));
    text-align: left;
  }
</style>
