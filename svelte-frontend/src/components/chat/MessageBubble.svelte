<script lang="ts">
  import type { UiMessage } from "../../lib/stores/messages.svelte";
  import { formatTime } from "../../lib/util/time";
  import { isGroup, jidUser, normalizeJid } from "../../lib/util/jid";
  import { contactFor, ensureContact } from "../../lib/stores/contacts.svelte";
  import { applyReaction } from "../../lib/stores/messages.svelte";
  import { startReply } from "../../lib/stores/compose.svelte";
  import { session } from "../../lib/stores/session.svelte";
  import { api } from "../../lib/ipc";
  import { trackRead } from "../../lib/receipts";
  import EmojiPicker from "./EmojiPicker.svelte";
  import MessageMedia from "./MessageMedia.svelte";

  let { message, group }: { message: UiMessage; group: boolean } = $props();
  let showReact = $state(false);

  async function react(emoji: string) {
    showReact = false;
    const me = normalizeJid(session.jid ?? "");
    // Toggle off if we already reacted with this same emoji.
    const current = me ? message.reactions?.[me] : undefined;
    const next = current === emoji ? "" : emoji;
    applyReaction(message.chatJid, message.id, me, next); // optimistic
    try {
      await api.sendReaction(
        message.chatJid,
        message.id,
        message.fromMe,
        next,
        isGroup(message.chatJid) ? message.senderJid : null,
      );
    } catch (e) {
      console.error("react failed", e);
    }
  }

  // In groups, resolve the sender to a display name: verified/contact name →
  // pushName → JID user. Trigger a lazy contact fetch the first time we render.
  $effect(() => {
    if (group && !message.fromMe && message.senderJid) void ensureContact(message.senderJid);
  });
  const senderName = $derived(
    contactFor(message.senderJid)?.name ?? message.pushName ?? jidUser(message.senderJid),
  );

  // Who the quoted message is from, for the reply preview header.
  const quotedSender = $derived.by(() => {
    const q = message.quoted;
    if (!q || !q.senderJid) return "";
    if (session.jid && normalizeJid(q.senderJid) === normalizeJid(session.jid)) return "You";
    return contactFor(q.senderJid)?.name ?? jidUser(q.senderJid);
  });

  // Summarize reactions into [emoji, count] pairs for the chip row.
  const reactionSummary = $derived.by(() => {
    const counts = new Map<string, number>();
    for (const e of Object.values(message.reactions ?? {})) {
      if (e) counts.set(e, (counts.get(e) ?? 0) + 1);
    }
    return [...counts.entries()];
  });
</script>

<div
  class="row"
  class:me={message.fromMe}
  use:trackRead={{
    chatJid: message.chatJid,
    id: message.id,
    senderJid: isGroup(message.chatJid) ? message.senderJid : null,
    fromMe: message.fromMe,
  }}
>
  <div class="bubble" class:me={message.fromMe}>
    {#if group && !message.fromMe}
      <div class="sender">{senderName}</div>
    {/if}
    {#if message.quoted && !message.deleted}
      <div class="quoted">
        {#if quotedSender}<div class="q-sender">{quotedSender}</div>{/if}
        <div class="q-text">{message.quoted.text ?? `[${message.quoted.kind}]`}</div>
      </div>
    {/if}
    {#if message.deleted}
      <div class="text deleted">🚫 This message was deleted</div>
    {:else}
      {#if message.media}
        <MessageMedia media={message.media} thumbnail={message.thumbnail} />
      {:else if message.thumbnail}
        <img class="thumb" src={`data:image/jpeg;base64,${message.thumbnail}`} alt="" />
      {/if}
      {#if message.text}
        <div class="text">{message.text}</div>
      {:else if !message.thumbnail && !message.media}
        <div class="text muted">[{message.kind}]</div>
      {/if}
    {/if}
    {#if reactionSummary.length}
      <div class="reactions">
        {#each reactionSummary as [emoji, count] (emoji)}
          <span class="reaction">{emoji}{#if count > 1}<span class="rcount">{count}</span>{/if}</span>
        {/each}
      </div>
    {/if}
    {#if !message.deleted}
      <div class="actions">
        <button class="react-btn" aria-label="Reply" onclick={() => startReply(message)}>↩</button>
        <div class="react-wrap">
          <button class="react-btn" aria-label="React" onclick={() => (showReact = !showReact)}
            >🙂</button
          >
          {#if showReact}
            <div class="react-pop" class:me={message.fromMe}>
              <EmojiPicker onpick={react} onclose={() => (showReact = false)} />
            </div>
          {/if}
        </div>
      </div>
    {/if}
    <div class="meta">
      {#if message.editedAt && !message.deleted}<span class="edited">edited</span>{/if}
      <span>{formatTime(message.timestamp)}</span>
      {#if message.fromMe}
        {#if message.status === "sending"}
          <span class="tick">🕓</span>
        {:else if message.status === "read" || message.status === "played"}
          <span class="tick read">✓✓</span>
        {:else if message.status === "delivered"}
          <span class="tick">✓✓</span>
        {:else}
          <span class="tick">✓</span>
        {/if}
      {/if}
    </div>
  </div>
</div>

<style>
  .row {
    display: flex;
    margin: 2px 0;
  }
  .row.me {
    justify-content: flex-end;
  }
  .bubble {
    position: relative;
    max-width: 65%;
    padding: 6px 9px 4px;
    border-radius: 8px;
    background: var(--wa-bubble-in);
    box-shadow: 0 1px 0.5px #0003;
  }
  .actions {
    position: absolute;
    top: -12px;
    right: -6px;
    display: flex;
    gap: 2px;
  }
  .react-wrap {
    position: relative;
  }
  .quoted {
    border-left: 3px solid var(--wa-green);
    background: rgba(0, 0, 0, 0.18);
    border-radius: 4px;
    padding: 3px 8px;
    margin-bottom: 4px;
    max-width: 100%;
  }
  .q-sender {
    font-size: 12px;
    font-weight: 600;
    color: var(--wa-green);
  }
  .q-text {
    font-size: 13px;
    color: var(--wa-text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .react-btn {
    opacity: 0;
    border: none;
    border-radius: 50%;
    width: 24px;
    height: 24px;
    font-size: 13px;
    background: var(--wa-panel);
    border: 1px solid var(--wa-border);
    transition: opacity 0.12s;
  }
  .row:hover .react-btn {
    opacity: 1;
  }
  .react-pop {
    position: absolute;
    top: 28px;
    right: 0;
    z-index: 20;
  }
  .react-pop.me {
    right: auto;
    left: 0;
  }
  .bubble.me {
    background: var(--wa-bubble-out);
  }
  .sender {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--wa-green);
    margin-bottom: 2px;
  }
  .thumb {
    max-width: 100%;
    border-radius: 6px;
    margin-bottom: 4px;
    display: block;
  }
  .text {
    font-size: 14.2px;
    line-height: 1.35;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .text.muted {
    color: var(--wa-text-muted);
    font-style: italic;
  }
  .text.deleted {
    color: var(--wa-text-muted);
    font-style: italic;
  }
  .reactions {
    display: flex;
    gap: 3px;
    margin-top: 3px;
  }
  .reaction {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    background: var(--wa-panel);
    border: 1px solid var(--wa-border);
    border-radius: 10px;
    padding: 0 5px;
    font-size: 12px;
    line-height: 18px;
  }
  .rcount {
    font-size: 10px;
    color: var(--wa-text-muted);
  }
  .edited {
    font-style: italic;
  }
  .meta {
    display: flex;
    justify-content: flex-end;
    gap: 4px;
    font-size: 11px;
    color: var(--wa-text-muted);
    margin-top: 2px;
  }
  .tick.read {
    color: #53bdeb;
  }
</style>
