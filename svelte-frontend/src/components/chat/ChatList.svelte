<script lang="ts">
  import { sortedChats } from "../../lib/stores/chats.svelte";
  import { contactFor } from "../../lib/stores/contacts.svelte";
  import { session } from "../../lib/stores/session.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import { displayName, jidUser } from "../../lib/util/jid";
  import ChatListItem from "./ChatListItem.svelte";

  let query = $state("");

  const list = $derived.by(() => {
    const all = sortedChats();
    const q = query.trim().toLowerCase();
    if (!q) return all;
    return all.filter((c) => {
      const name = displayName(c.jid, c.name ?? contactFor(c.jid)?.name).toLowerCase();
      return (
        name.includes(q) ||
        jidUser(c.jid).toLowerCase().includes(q) ||
        (c.lastMessage ?? "").toLowerCase().includes(q)
      );
    });
  });
</script>

<div class="sidebar">
  <header>
    <span class="me">{session.jid ? jidUser(session.jid) : "WhatsApp"}</span>
    <button class="settings" aria-label="Settings" onclick={() => (ui.settingsOpen = true)}
      >⚙</button
    >
  </header>
  <div class="search">
    <input type="search" placeholder="Search chats" bind:value={query} />
  </div>
  <div class="chats">
    {#each list as chat (chat.jid)}
      <ChatListItem {chat} />
    {/each}
    {#if list.length === 0}
      <p class="empty">
        {query.trim()
          ? "No chats match your search."
          : "No chats yet. History loads after pairing; new messages appear live."}
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
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 14px;
    background: var(--wa-panel);
    font-weight: 600;
  }
  .settings {
    border: none;
    background: transparent;
    color: var(--wa-text-muted);
    font-size: 20px;
    line-height: 1;
  }
  .settings:hover {
    color: var(--wa-text);
  }
  .search {
    padding: 8px 12px;
    background: var(--wa-bg);
    border-bottom: 1px solid var(--wa-border);
    flex-shrink: 0;
  }
  .search input {
    width: 100%;
    padding: 8px 12px;
    border: none;
    border-radius: 8px;
    background: var(--wa-panel-2);
    color: var(--wa-text);
    font-size: 13px;
  }
  .search input:focus {
    outline: none;
  }
  .chats {
    flex: 1;
    /* Without this a flex child defaults to min-height:auto and grows to fit its
       content instead of scrolling, so the list overflows the panel. */
    min-height: 0;
    overflow-y: auto;
  }
  .empty {
    color: var(--wa-text-muted);
    font-size: 13px;
    padding: 20px;
    text-align: center;
  }
</style>
