<script lang="ts">
  import { archivedChats, sortedChats } from "../../lib/stores/chats.svelte";
  import { contactFor } from "../../lib/stores/contacts.svelte";
  import { session } from "../../lib/stores/session.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import { displayName, jidUser } from "../../lib/util/jid";
  import ChatListItem from "./ChatListItem.svelte";

  let query = $state("");
  let viewingArchived = $state(false);

  const archived = $derived(archivedChats());

  // Bounce back to the main list if the last archived chat gets unarchived.
  $effect(() => {
    if (viewingArchived && archived.length === 0) viewingArchived = false;
  });

  const list = $derived.by(() => {
    const base = viewingArchived ? archived : sortedChats();
    const q = query.trim().toLowerCase();
    if (!q) return base;
    return base.filter((c) => {
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
    {#if viewingArchived}
      <button class="back" aria-label="Back" onclick={() => (viewingArchived = false)}>←</button>
      <span class="me">Archived</span>
    {:else}
      <span class="me">{session.jid ? jidUser(session.jid) : "WhatsApp"}</span>
      <button class="settings" aria-label="Settings" onclick={() => (ui.settingsOpen = true)}
        >⚙</button
      >
    {/if}
  </header>
  <div class="search">
    <input
      type="search"
      placeholder={viewingArchived ? "Search archived" : "Search chats"}
      bind:value={query}
    />
  </div>
  <div class="chats">
    {#if !viewingArchived && !query.trim() && archived.length}
      <button class="archived-entry" onclick={() => (viewingArchived = true)}>
        <span class="arch-icon">🗄</span>
        <span class="arch-label">Archived</span>
        <span class="arch-count">{archived.length}</span>
      </button>
    {/if}
    {#each list as chat (chat.jid)}
      <ChatListItem {chat} />
    {/each}
    {#if list.length === 0}
      <p class="empty">
        {#if query.trim()}
          No chats match your search.
        {:else if viewingArchived}
          No archived chats.
        {:else}
          No chats yet. History loads after pairing; new messages appear live.
        {/if}
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
  .me {
    flex: 1;
    min-width: 0;
  }
  .settings,
  .back {
    border: none;
    background: transparent;
    color: var(--wa-text-muted);
    font-size: 20px;
    line-height: 1;
  }
  .settings:hover,
  .back:hover {
    color: var(--wa-text);
  }
  .back {
    margin-right: 12px;
  }
  .archived-entry {
    display: flex;
    align-items: center;
    gap: 14px;
    width: 100%;
    padding: 12px 18px;
    background: none;
    border: none;
    border-bottom: 1px solid var(--wa-border);
    color: var(--wa-text);
    text-align: left;
    font-size: 15px;
  }
  .archived-entry:hover {
    background: var(--wa-hover);
  }
  .arch-icon {
    font-size: 16px;
    color: var(--wa-text-muted);
  }
  .arch-label {
    flex: 1;
  }
  .arch-count {
    color: var(--wa-text-muted);
    font-size: 13px;
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
