<script lang="ts">
  import { sortedChats } from "../../lib/stores/chats.svelte";
  import { session } from "../../lib/stores/session.svelte";
  import { jidUser } from "../../lib/util/jid";
  import ChatListItem from "./ChatListItem.svelte";

  const list = $derived(sortedChats());
</script>

<div class="sidebar">
  <header>
    <span class="me">{session.jid ? jidUser(session.jid) : "WhatsApp"}</span>
  </header>
  <div class="chats">
    {#each list as chat (chat.jid)}
      <ChatListItem {chat} />
    {/each}
    {#if list.length === 0}
      <p class="empty">
        No chats yet. History loads after pairing; new messages appear live.
      </p>
    {/if}
  </div>
</div>

<style>
  .sidebar {
    height: 100%;
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--wa-border);
    background: var(--wa-bg);
  }
  header {
    padding: 16px 14px;
    background: var(--wa-panel);
    font-weight: 600;
  }
  .chats {
    flex: 1;
    overflow-y: auto;
  }
  .empty {
    color: var(--wa-text-muted);
    font-size: 13px;
    padding: 20px;
    text-align: center;
  }
</style>
