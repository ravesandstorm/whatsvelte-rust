<script lang="ts">
  import type { Chat } from "../../lib/stores/chats.svelte";
  import { chatUi, selectChat, setChatFlag } from "../../lib/stores/chats.svelte";
  import { contactFor } from "../../lib/stores/contacts.svelte";
  import { api } from "../../lib/ipc";
  import { displayName, isGroup } from "../../lib/util/jid";
  import { formatChatTime } from "../../lib/util/time";
  import Avatar from "../common/Avatar.svelte";

  let { chat }: { chat: Chat } = $props();
  const name = $derived(displayName(chat.jid, chat.name ?? contactFor(chat.jid)?.name));

  let menu = $state<{ x: number; y: number } | null>(null);

  function openMenu(e: MouseEvent) {
    e.preventDefault();
    menu = { x: e.clientX, y: e.clientY };
  }

  async function toggle(
    flag: "muted" | "pinned" | "archived",
    call: (jid: string, v: boolean) => Promise<void>,
  ) {
    const next = !chat[flag];
    menu = null;
    setChatFlag(chat.jid, flag, next); // optimistic; server echo confirms
    try {
      await call(chat.jid, next);
    } catch (err) {
      console.error(`set ${flag} failed`, err);
      setChatFlag(chat.jid, flag, !next); // revert on failure
    }
  }
</script>

<button
  class="item"
  class:active={chatUi.activeJid === chat.jid}
  onclick={() => selectChat(chat.jid)}
  oncontextmenu={openMenu}
>
  <Avatar label={name} jid={chat.jid} group={isGroup(chat.jid)} />
  <div class="mid">
    <div class="top">
      <span class="name">{name}</span>
      <span class="icons">
        {#if chat.pinned}<span title="Pinned">📌</span>{/if}
        {#if chat.muted}<span title="Muted">🔇</span>{/if}
      </span>
      <span class="time">{formatChatTime(chat.timestamp)}</span>
    </div>
    <div class="bottom">
      <span class="preview">{chat.archived ? "🗄 " : ""}{chat.lastMessage ?? ""}</span>
      {#if chat.unread}<span class="badge">{chat.unread}</span>{/if}
    </div>
  </div>
</button>

{#if menu}
  <div class="menu-backdrop" onclick={() => (menu = null)} role="presentation"></div>
  <div class="menu" style:left={`${menu.x}px`} style:top={`${menu.y}px`}>
    <button onclick={() => toggle("pinned", api.setChatPinned)}>
      {chat.pinned ? "Unpin" : "Pin"} chat
    </button>
    <button onclick={() => toggle("muted", api.setChatMuted)}>
      {chat.muted ? "Unmute" : "Mute"} chat
    </button>
    <button onclick={() => toggle("archived", api.setChatArchived)}>
      {chat.archived ? "Unarchive" : "Archive"} chat
    </button>
  </div>
{/if}

<style>
  .item {
    display: flex;
    gap: 12px;
    align-items: center;
    width: 100%;
    padding: 10px 14px;
    background: none;
    border: none;
    border-bottom: 1px solid var(--wa-border);
    text-align: left;
    color: var(--wa-text);
  }
  .item:hover {
    background: var(--wa-hover);
  }
  .item.active {
    background: var(--wa-panel-2);
  }
  .mid {
    flex: 1;
    min-width: 0;
  }
  .top {
    display: flex;
    align-items: baseline;
    gap: 6px;
  }
  .name {
    flex: 1;
    min-width: 0;
    font-size: 15px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .icons {
    display: flex;
    gap: 3px;
    font-size: 12px;
    flex-shrink: 0;
    margin-left: auto;
  }
  .time {
    font-size: 12px;
    color: var(--wa-text-muted);
    flex-shrink: 0;
  }
  .menu-backdrop {
    position: fixed;
    inset: 0;
    z-index: 40;
  }
  .menu {
    position: fixed;
    z-index: 41;
    background: var(--wa-panel-2);
    border: 1px solid var(--wa-border);
    border-radius: 8px;
    padding: 4px;
    box-shadow: 0 6px 18px rgba(0, 0, 0, 0.4);
    display: flex;
    flex-direction: column;
    min-width: 160px;
  }
  .menu button {
    border: none;
    background: transparent;
    color: var(--wa-text);
    text-align: left;
    padding: 8px 12px;
    border-radius: 6px;
    font-size: 14px;
  }
  .menu button:hover {
    background: var(--wa-hover);
  }
  .bottom {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
    margin-top: 2px;
  }
  .preview {
    font-size: 13px;
    color: var(--wa-text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .badge {
    background: var(--wa-unread);
    color: #04221c;
    font-size: 12px;
    font-weight: 700;
    min-width: 20px;
    height: 20px;
    border-radius: 10px;
    display: grid;
    place-items: center;
    padding: 0 6px;
    flex-shrink: 0;
  }
</style>
