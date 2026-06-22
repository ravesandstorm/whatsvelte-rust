<script lang="ts">
  import { messagesFor } from "../../lib/stores/messages.svelte";
  import { chats } from "../../lib/stores/chats.svelte";
  import { contactFor } from "../../lib/stores/contacts.svelte";
  import { session } from "../../lib/stores/session.svelte";
  import { wallpaperCss, wallpaperFor } from "../../lib/stores/settings.svelte";
  import { displayName, isGroup, isSelf } from "../../lib/util/jid";
  import Avatar from "../common/Avatar.svelte";
  import MessageList from "./MessageList.svelte";
  import MessageComposer from "./MessageComposer.svelte";

  let { jid }: { jid: string } = $props();
  const msgs = $derived(messagesFor(jid));
  const name = $derived.by(() => {
    // The "message yourself" chat: show our own profile name + (You).
    if (isSelf(jid, session.jid)) return `${session.pushName ?? displayName(jid)} (You)`;
    return displayName(jid, chats.get(jid)?.name ?? contactFor(jid)?.name);
  });
  const wallpaper = $derived(wallpaperCss(wallpaperFor(jid)));
</script>

<div class="conversation" style:background={wallpaper}>
  <header>
    <Avatar label={name} jid={jid} group={isGroup(jid)} size={40} />
    <div class="title">{name}</div>
  </header>
  <MessageList messages={msgs} chatJid={jid} />
  <MessageComposer {jid} />
</div>

<style>
  .conversation {
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 16px;
    background: var(--wa-panel);
    border-left: 1px solid var(--wa-border);
  }
  .title {
    font-size: 15px;
    font-weight: 500;
  }
</style>
