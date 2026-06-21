<script lang="ts">
  import type { UiMessage } from "../../lib/stores/messages.svelte";
  import { isGroup } from "../../lib/util/jid";
  import MessageBubble from "./MessageBubble.svelte";

  let { messages, chatJid }: { messages: UiMessage[]; chatJid: string } = $props();
  let el: HTMLDivElement | undefined = $state();

  // Auto-scroll to bottom on new messages / chat switch.
  $effect(() => {
    messages.length;
    chatJid;
    if (el) el.scrollTop = el.scrollHeight;
  });
</script>

<div class="list" bind:this={el}>
  {#each messages as m (m.id)}
    <MessageBubble message={m} group={isGroup(chatJid)} />
  {/each}
</div>

<style>
  .list {
    flex: 1;
    overflow-y: auto;
    padding: 14px 8%;
    background: var(--wa-chat-bg);
    display: flex;
    flex-direction: column;
  }
</style>
