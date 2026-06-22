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
    {#if view !== "main"}
      <button class="back" aria-label="Back" onclick={() => (view = "main")}>←</button>
      <span class="me">{titles[view]}</span>
    {:else}
      <span class="me">{session.pushName ?? (session.jid ? formatPhone(session.jid) : "WhatsApp")}</span>
      <button class="settings" aria-label="Settings" onclick={() => (ui.settingsOpen = true)}
        >⚙</button
      >
    {/if}
  </header>
  <div class="search">
    <input
      type="search"
      placeholder={view === "main" ? "Search chats" : `Search ${titles[view].toLowerCase()}`}
      bind:value={query}
    />
  </div>
  <div class="chats">
    {#if view === "main" && !query.trim()}
      {#each sections as s (s.view)}
        {#if s.count}
          <button class="archived-entry" onclick={() => (view = s.view)}>
            <span class="arch-icon">{s.icon}</span>
            <span class="arch-label">{s.label}</span>
            <span class="arch-count">{s.count}</span>
          </button>
        {/if}
      {/each}
    {/if}
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
