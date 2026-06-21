<script lang="ts">
  import { chatUi } from "../../lib/stores/chats.svelte";
  import ChatList from "./ChatList.svelte";
  import Conversation from "./Conversation.svelte";
</script>

<div class="layout">
  <aside><ChatList /></aside>
  <section>
    {#if chatUi.activeJid}
      {#key chatUi.activeJid}
        <Conversation jid={chatUi.activeJid} />
      {/key}
    {:else}
      <div class="empty">
        <h2>WhatSvelte Rust</h2>
        <p>Select a chat to start messaging.</p>
      </div>
    {/if}
  </section>
</div>

<style>
  .layout {
    height: 100%;
    display: grid;
    grid-template-columns: 360px 1fr;
  }
  section {
    min-width: 0;
    background: var(--wa-chat-bg);
  }
  .empty {
    height: 100%;
    display: grid;
    place-content: center;
    text-align: center;
    color: var(--wa-text-muted);
  }
  .empty h2 {
    color: var(--wa-green);
    font-weight: 500;
  }
</style>
