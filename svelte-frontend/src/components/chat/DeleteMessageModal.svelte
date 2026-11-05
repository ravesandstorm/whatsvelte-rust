<script lang="ts">
  import {
    applyRevoke,
    isPending,
    protoChatJid,
    protoSenderJid,
    removeMessage,
    restoreMessage,
    type UiMessage,
  } from "../../lib/stores/messages.svelte";
  import { showToast } from "../../lib/stores/toast.svelte";
  import { isGroup } from "../../lib/util/jid";
  import { api } from "../../lib/ipc";

  let { message, onclose }: { message: UiMessage; onclose: () => void } = $props();

  // A bubble the server never accepted exists only here: dismiss it locally.
  const localOnly = $derived(isPending(message));
  // Revoke-for-everyone only makes sense for our own, not-yet-deleted messages.
  const canDeleteEveryone = $derived(message.fromMe && !message.deleted && !localOnly);

  async function deleteForMe() {
    onclose();
    const before = message;
    removeMessage(before.chatJid, before.id); // optimistic — no tombstone
    if (localOnly) return;
    try {
      await api.deleteForMe(
        protoChatJid(before),
        before.id,
        before.fromMe,
        isGroup(before.chatJid) && !before.fromMe ? protoSenderJid(before) : null,
        before.timestamp,
      );
    } catch (e) {
      console.error("delete for me failed", e);
      restoreMessage(before);
      showToast("Couldn't delete the message");
    }
  }

  async function deleteForEveryone() {
    onclose();
    const before = message;
    applyRevoke(before.chatJid, before.id); // optimistic tombstone
    try {
      await api.revokeMessage(protoChatJid(before), before.id, null);
    } catch (e) {
      console.error("revoke failed", e);
      restoreMessage(before);
      showToast("Couldn't delete the message for everyone");
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onclose()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onclose} role="presentation">
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div
    class="modal"
    role="dialog"
    aria-modal="true"
    aria-label="Delete message"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
  >
    <h3 class="title">Delete message?</h3>
    <div class="actions">
      {#if canDeleteEveryone}
        <button class="opt danger" onclick={deleteForEveryone}>Delete for everyone</button>
      {/if}
      <button class="opt danger" onclick={deleteForMe}>Delete for me</button>
      <button class="opt" onclick={onclose}>Cancel</button>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 1100;
    background: rgba(0, 0, 0, 0.55);
    display: grid;
    place-items: center;
  }
  .modal {
    width: min(420px, 90vw);
    background: var(--wa-panel-2);
    border-radius: 12px;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.5);
    padding: 22px 24px 16px;
  }
  .title {
    margin: 0 0 18px;
    font-size: 16px;
    font-weight: 500;
    color: var(--wa-text);
  }
  .actions {
    display: flex;
    flex-direction: column;
    gap: 4px;
    align-items: flex-end;
  }
  .opt {
    border: none;
    background: transparent;
    color: var(--wa-accent, #00a884);
    text-transform: uppercase;
    font-size: 13px;
    font-weight: 600;
    letter-spacing: 0.4px;
    padding: 10px 18px;
    border-radius: 6px;
    cursor: pointer;
    width: 100%;
    text-align: center;
  }
  .opt:hover {
    background: var(--wa-highlight);
  }
  .opt.danger {
    color: #e06457;
  }
</style>
