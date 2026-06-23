<script lang="ts">
  import { chatUi } from "../../lib/stores/chats.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import ChatList from "./ChatList.svelte";
  import Conversation from "./Conversation.svelte";
  import MediaLightbox from "./MediaLightbox.svelte";
  import SettingsPanel from "../settings/SettingsPanel.svelte";
</script>

{#if ui.settingsOpen}
  <SettingsPanel />
{/if}

{#if ui.lightboxMedia}
  <MediaLightbox />
{/if}

<div class="layout">
  <aside class="pane"><ChatList /></aside>
  <section class="pane">
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
    /* Pin the single row to the viewport so panes get a definite height to
       scroll within, instead of growing with their content. */
    grid-template-rows: 100%;
  }
  /* Both grid cells must clip + allow shrinking so their inner scroll regions
     (chat list, message list) actually scroll rather than overflow the window. */
  .pane {
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }
  section {
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
