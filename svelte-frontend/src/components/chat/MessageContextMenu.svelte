<script lang="ts">
  import type { UiMessage } from "../../lib/stores/messages.svelte";
  import { startReply, startEdit } from "../../lib/stores/compose.svelte";

  let {
    x,
    y,
    message,
    onclose,
    ondelete,
  }: {
    x: number;
    y: number;
    message: UiMessage;
    onclose: () => void;
    ondelete: () => void;
  } = $props();

  const canReply = $derived(!message.deleted);
  const canEdit = $derived(message.fromMe && !!message.text && !message.media && !message.deleted);
  const canDelete = $derived(!message.deleted);

  // Keep the menu on-screen (approximate menu box ~ 210x150).
  const px = $derived(Math.max(8, Math.min(x, window.innerWidth - 214)));
  const py = $derived(Math.max(8, Math.min(y, window.innerHeight - 160)));

  function reply() {
    startReply(message);
    onclose();
  }

  function edit() {
    startEdit(message);
    onclose();
  }

  // Hand off to the parent's confirmation modal (which offers
  // delete-for-me / delete-for-everyone).
  function del() {
    ondelete();
  }
</script>

<svelte:window
  onclick={onclose}
  oncontextmenu={onclose}
  onkeydown={(e) => e.key === "Escape" && onclose()}
/>

<div class="menu" style="left:{px}px; top:{py}px" role="menu" tabindex="-1">
  {#if canReply}
    <button role="menuitem" onclick={reply}>Reply</button>
  {/if}
  {#if canEdit}
    <button role="menuitem" onclick={edit}>Edit</button>
  {/if}
  {#if canDelete}
    <button role="menuitem" class="danger" onclick={del}>Delete</button>
  {/if}
</div>

<style>
  .menu {
    position: fixed;
    z-index: 1000;
    min-width: 190px;
    padding: 5px;
    background: var(--wa-panel-2);
    border: 1px solid var(--wa-border);
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
    display: flex;
    flex-direction: column;
  }
  .menu button {
    border: none;
    background: transparent;
    color: var(--wa-text);
    text-align: left;
    padding: 8px 12px;
    border-radius: 6px;
    font-size: 14px;
    cursor: pointer;
  }
  .menu button:hover {
    background: var(--wa-highlight);
  }
  .menu button.danger {
    color: #e06457;
  }
</style>
