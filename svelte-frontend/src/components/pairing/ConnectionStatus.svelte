<script lang="ts">
  import { session } from "../../lib/stores/session.svelte";
</script>

<div class="status" class:on={session.connected}>
  <span class="dot"></span>
  {session.connected ? "Connected" : "Connecting…"}
</div>

<style>
  .status {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--wa-text-muted);
  }
  .dot {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: #c8a415;
  }
  .status.on .dot {
    background: var(--wa-green);
  }
</style>
