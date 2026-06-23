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

  // `always` sections (Channels, Status) are top-level navigation and stay
  // visible even when empty — otherwise the buttons flicker/vanish as counts
  // settle during async hydration, and the user has no way to reach the section.
  // Archived only appears when there's something archived (WhatsApp behaviour).
  const sections: { view: View; icon: string; label: string; count: number; always: boolean }[] =
    $derived([
      { view: "channels", icon: "📢", label: "Channels", count: channels.length, always: true },
      { view: "status", icon: "⭕", label: "Status", count: statuses.length, always: true },
      { view: "archived", icon: "🗄", label: "Archived", count: archived.length, always: false },
    ]);

  const titles: Record<View, string> = {
    main: "",
    archived: "Archived",
    channels: "Channels",
    status: "Status",
  };

  // Bounce back to the main list only if the Archived section (the one that can
  // disappear) becomes empty while open. Channels/Status stay reachable even
  // when empty, so they show their own empty state instead of bouncing.
  $effect(() => {
    if (view === "archived" && archived.length === 0) view = "main";
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
    <div class="left">
      <span class="me">{session.pushName ?? (session.jid ? formatPhone(session.jid) : "WhatsApp")}</span>
      {#if session.historySyncing}
        <span class="syncing" title="Syncing chat history…">
          <span class="sync-dot"></span>Syncing…
        </span>
      {/if}
    </div>
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
      {#if s.always || s.count}
        <button class="pill" class:active={view === s.view} onclick={() => (view = s.view)}>
          <span class="picon">{s.icon}</span>{s.label}{#if s.count}<span class="pcount">{s.count}</span>{/if}
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
  .left {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .me {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .syncing {
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    font-weight: 400;
    color: var(--wa-text-muted);
  }
  .sync-dot {
    width: 11px;
    height: 11px;
    border: 1.5px solid var(--wa-border);
    border-top-color: var(--wa-green);
    border-radius: 50%;
    animation: sync-spin 0.8s linear infinite;
  }
  @keyframes sync-spin {
    to {
      transform: rotate(360deg);
    }
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
