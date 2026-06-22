<script lang="ts">
  import type { Chat } from "../../lib/stores/chats.svelte";
  import { chatUi, selectChat, setChatFlag } from "../../lib/stores/chats.svelte";
  import { contactFor } from "../../lib/stores/contacts.svelte";
  import { messagesFor, type MessageStatus } from "../../lib/stores/messages.svelte";
  import { api } from "../../lib/ipc";
  import { displayName, formatPhone, isGroup } from "../../lib/util/jid";
  import { mediaLabel } from "../../lib/util/preview";
  import { formatChatTime } from "../../lib/util/time";
  import Avatar from "../common/Avatar.svelte";

  let { chat }: { chat: Chat } = $props();
  const group = $derived(isGroup(chat.jid));
  const name = $derived(displayName(chat.jid, chat.name ?? contactFor(chat.jid)?.name));

  // Derive the preview from the chat's latest message (it already carries
  // sender/fromMe/kind/status) so we get ticks, a "You:"/sender prefix and media
  // labels for free; fall back to the chat row's stored text when no message is
  // loaded.
  const last = $derived.by(() => {
    const arr = messagesFor(chat.jid);
    return arr.length ? arr[arr.length - 1] : null;
  });
  const preview = $derived.by(() => {
    const m = last;
    if (!m)
      return {
        status: null as MessageStatus | null,
        prefix: "",
        body: chat.lastMessage ?? "",
        full: chat.lastMessage ?? "",
      };
    const body = m.deleted ? "🚫 This message was deleted" : (m.text ?? mediaLabel(m.kind));
    let prefix = "";
    if (m.fromMe) prefix = group ? "You: " : "";
    else if (group) {
      const sn = contactFor(m.senderJid)?.name ?? m.pushName ?? formatPhone(m.senderJid);
      prefix = sn ? `${sn}: ` : "";
    }
    return {
      status: m.fromMe ? (m.status ?? "sent") : null,
      prefix,
      body,
      full: m.text ?? body,
    };
  });
  const tickGlyph = $derived(preview.status === "sending" ? "🕓" : preview.status === "sent" ? "✓" : "✓✓");
  const tickRead = $derived(preview.status === "read" || preview.status === "played");

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
  <Avatar label={name} jid={chat.jid} group={group} />
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
      <span class="preview" title={preview.full}>
        {#if preview.status}<span class="tick" class:read={tickRead}>{tickGlyph}</span> {/if}{preview.prefix}{preview.body}
      </span>
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
  .tick {
    font-size: 12px;
  }
  .tick.read {
    color: #53bdeb;
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
