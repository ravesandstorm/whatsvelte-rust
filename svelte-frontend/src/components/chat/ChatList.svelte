<script lang="ts">
  import {
    archivedChats,
    channelChats,
    sortedChats,
    statusChats,
  } from "../../lib/stores/chats.svelte";
  import { contactFor } from "../../lib/stores/contacts.svelte";
  import { session } from "../../lib/stores/session.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import { displayName, formatPhone, jidUser } from "../../lib/util/jid";
  import ChatListItem from "./ChatListItem.svelte";

  type View = "main" | "archived" | "channels" | "status";

  let query = $state("");
  let view = $state<View>("main");

  const archived = $derived(archivedChats());
  const channels = $derived(channelChats());
  const statuses = $derived(statusChats());

  const sections: { view: View; icon: string; label: string; count: number }[] = $derived([
    { view: "channels", icon: "📢", label: "Channels", count: channels.length },
    { view: "status", icon: "⭕", label: "Status", count: statuses.length },
    { view: "archived", icon: "🗄", label: "Archived", count: archived.length },
  ]);

  const titles: Record<View, string> = {
    main: "",
    archived: "Archived",
    channels: "Channels",
    status: "Status",
  };

  // Bounce back to the main list if the section we're viewing becomes empty.
  $effect(() => {
    if (view === "archived" && archived.length === 0) view = "main";
    if (view === "channels" && channels.length === 0) view = "main";
    if (view === "status" && statuses.length === 0) view = "main";
  });

  const list = $derived.by(() => {
    const base =
      view === "archived"
        ? archived
        : view === "channels"
          ? channels
          : view === "status"
            ? statuses
            : sortedChats();
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
    <span class="me">{session.pushName ?? (session.jid ? formatPhone(session.jid) : "WhatsApp")}</span>
    <button class="settings" aria-label="Settings" onclick={() => (ui.settingsOpen = true)}>⚙</button>
  </header>
  <div class="search">
    <input
      type="search"
      placeholder={view === "main" ? "Search chats" : `Search ${titles[view].toLowerCase()}`}
      bind:value={query}
    />
  </div>
  <div class="filters">
    <button class="pill" class:active={view === "main"} onclick={() => (view = "main")}>All</button>
    {#each sections as s (s.view)}
      {#if s.count}
        <button class="pill" class:active={view === s.view} onclick={() => (view = s.view)}>
          <span class="picon">{s.icon}</span>{s.label}<span class="pcount">{s.count}</span>
        </button>
      {/if}
    {/each}
  </div>
  <div class="chats">
    {#each list as chat (chat.jid)}
      <ChatListItem {chat} />
    {/each}
    {#if list.length === 0}
      <p class="empty">
        {#if query.trim()}
          No chats match your search.
        {:else if view !== "main"}
          No {titles[view].toLowerCase()} yet.
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
  .filters {
    display: flex;
    gap: 8px;
    padding: 8px 12px;
    overflow-x: auto;
    background: var(--wa-bg);
    border-bottom: 1px solid var(--wa-border);
    flex-shrink: 0;
  }
  .filters::-webkit-scrollbar {
    height: 0;
  }
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    flex: 0 0 auto;
    border: 1px solid var(--wa-border);
    background: var(--wa-panel-2);
    color: var(--wa-text-muted);
    padding: 5px 12px;
    border-radius: 16px;
    font-size: 13px;
    white-space: nowrap;
  }
  .pill:hover {
    background: var(--wa-hover);
  }
  .pill.active {
    background: var(--wa-green);
    border-color: var(--wa-green);
    color: #04221c;
  }
  .picon {
    font-size: 13px;
  }
  .pcount {
    font-size: 11px;
    font-weight: 600;
    background: var(--wa-panel);
    color: var(--wa-text-muted);
    border-radius: 9px;
    padding: 0 6px;
    min-width: 16px;
    text-align: center;
  }
  .pill.active .pcount {
    background: rgba(0, 0, 0, 0.22);
    color: #04221c;
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
