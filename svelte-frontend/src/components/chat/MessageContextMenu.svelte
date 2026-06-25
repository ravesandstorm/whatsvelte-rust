<script lang="ts">
  import type { UiMessage } from "../../lib/stores/messages.svelte";
  import { applyRevoke, removeMessage } from "../../lib/stores/messages.svelte";
  import { startReply, startEdit } from "../../lib/stores/compose.svelte";
  import { isGroup } from "../../lib/util/jid";
  import { api } from "../../lib/ipc";

  let {
    x,
    y,
    message,
    onclose,
  }: { x: number; y: number; message: UiMessage; onclose: () => void } = $props();

  const canReply = $derived(!message.deleted);
  const canEdit = $derived(message.fromMe && !!message.text && !message.media && !message.deleted);
  // Revoke-for-everyone only makes sense for our own, not-yet-deleted messages.
  const canDeleteEveryone = $derived(message.fromMe && !message.deleted);

  // Keep the menu on-screen (approximate menu box ~ 210x190).
  const px = $derived(Math.max(8, Math.min(x, window.innerWidth - 214)));
  const py = $derived(Math.max(8, Math.min(y, window.innerHeight - 196)));

  function reply() {
    startReply(message);
    onclose();
  }

  function edit() {
    startEdit(message);
    onclose();
  }

  async function deleteForMe() {
    onclose();
    removeMessage(message.chatJid, message.id); // optimistic
    try {
      await api.deleteForMe(
        message.chatJid,
        message.id,
        message.fromMe,
        isGroup(message.chatJid) && !message.fromMe ? message.senderJid : null,
        message.timestamp,
      );
    } catch (e) {
      console.error("delete for me failed", e);
    }
  }

  async function deleteForEveryone() {
    onclose();
    applyRevoke(message.chatJid, message.id); // optimistic tombstone
    try {
      await api.revokeMessage(message.chatJid, message.id, null);
    } catch (e) {
      console.error("revoke failed", e);
    }
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
  <button role="menuitem" onclick={deleteForMe}>Delete for me</button>
  {#if canDeleteEveryone}
    <button role="menuitem" class="danger" onclick={deleteForEveryone}>Delete for everyone</button>
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
    background: var(--wa-hover);
  }
  .menu button.danger {
    color: #e06457;
  }
</style>
