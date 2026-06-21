<script lang="ts">
  import type { Chat } from "../../lib/stores/chats.svelte";
  import { chatUi, selectChat } from "../../lib/stores/chats.svelte";
  import { contactFor } from "../../lib/stores/contacts.svelte";
  import { displayName, isGroup } from "../../lib/util/jid";
  import { formatChatTime } from "../../lib/util/time";
  import Avatar from "../common/Avatar.svelte";

  let { chat }: { chat: Chat } = $props();
  const name = $derived(displayName(chat.jid, chat.name ?? contactFor(chat.jid)?.name));
</script>

<button class="item" class:active={chatUi.activeJid === chat.jid} onclick={() => selectChat(chat.jid)}>
  <Avatar label={name} jid={chat.jid} group={isGroup(chat.jid)} />
  <div class="mid">
    <div class="top">
      <span class="name">{name}</span>
      <span class="time">{formatChatTime(chat.timestamp)}</span>
    </div>
    <div class="bottom">
      <span class="preview">{chat.lastMessage ?? ""}</span>
      {#if chat.unread}<span class="badge">{chat.unread}</span>{/if}
    </div>
  </div>
</button>

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
    justify-content: space-between;
    align-items: baseline;
    gap: 8px;
  }
  .name {
    font-size: 15px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .time {
    font-size: 12px;
    color: var(--wa-text-muted);
    flex-shrink: 0;
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
