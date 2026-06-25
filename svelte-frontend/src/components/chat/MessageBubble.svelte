<script lang="ts">
  import type { UiMessage } from "../../lib/stores/messages.svelte";
  import { formatTime } from "../../lib/util/time";
  import { formatPhone, isGroup, normalizeJid } from "../../lib/util/jid";
  import { contactFor, ensureContact } from "../../lib/stores/contacts.svelte";
  import { canonicalJid } from "../../lib/stores/lid";
  import { applyReaction } from "../../lib/stores/messages.svelte";
  import { startEdit, startReply } from "../../lib/stores/compose.svelte";
  import { scrollToMessage } from "../../lib/stores/scroll.svelte";
  import { session } from "../../lib/stores/session.svelte";
  import { api } from "../../lib/ipc";
  import { trackRead } from "../../lib/receipts";
  import ReactionPicker from "./ReactionPicker.svelte";
  import MessageMedia from "./MessageMedia.svelte";
  import MessageContextMenu from "./MessageContextMenu.svelte";

  let {
    message,
    group,
    prev = null,
    highlighted = false,
  }: { message: UiMessage; group: boolean; prev?: UiMessage | null; highlighted?: boolean } =
    $props();
  let showReact = $state(false);
  let menu = $state<{ x: number; y: number } | null>(null);

  // Consecutive messages from the same sender group together (tight spacing, no
  // repeated sender name); a different sender starts a new visual group.
  const sameSenderAsPrev = $derived(
    !!prev &&
      prev.fromMe === message.fromMe &&
      prev.senderJid === message.senderJid &&
      !prev.deleted,
  );
  // Stickers render borderless (no bubble chrome), WhatsApp-style.
  const bareSticker = $derived(message.kind === "sticker" && !!message.media && !message.deleted);

  function onContext(e: MouseEvent) {
    e.preventDefault();
    menu = { x: e.clientX, y: e.clientY };
  }

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

  // Group participants may be addressed by a `@lid`; resolve to the canonical
  // (PN) form so contact/name lookups and phone formatting work.
  const resolvedSender = $derived(canonicalJid(message.senderJid));

  // In groups, resolve the sender to a display name: verified/contact name →
  // pushName → formatted phone. Trigger a lazy contact fetch the first time we
  // render.
  $effect(() => {
    if (group && !message.fromMe && message.senderJid) void ensureContact(resolvedSender);
    // Resolve the quoted message's sender too, so the reply preview shows a name
    // rather than a bare number.
    const qs = message.quoted?.senderJid;
    if (qs && !(session.jid && normalizeJid(qs) === normalizeJid(session.jid)))
      void ensureContact(canonicalJid(qs));
  });
  const senderName = $derived(
    contactFor(resolvedSender)?.name ?? message.pushName ?? formatPhone(resolvedSender),
  );

  // Who the quoted message is from, for the reply preview header.
  const quotedSender = $derived.by(() => {
    const q = message.quoted;
    if (!q || !q.senderJid) return "";
    const qs = canonicalJid(q.senderJid);
    if (session.jid && normalizeJid(qs) === normalizeJid(session.jid)) return "You";
    return contactFor(qs)?.name ?? formatPhone(qs);
  });

  // Interactive / business message kinds get a small "type" label so the user
  // knows the bubble is more than plain text (rendered read-only).
  const STRUCTURED_LABELS: Record<string, string> = {
    buttons: "Interactive",
    list: "Interactive",
    interactive: "Interactive",
    template: "Interactive",
    poll: "Poll",
    order: "Order",
    product: "Product",
    contact: "Contact",
    location: "Location",
  };
  const typeLabel = $derived(STRUCTURED_LABELS[message.kind] ?? null);

  // Summarize reactions into [emoji, count] pairs for the chip row.
  const reactionSummary = $derived.by(() => {
    const counts = new Map<string, number>();
    for (const e of Object.values(message.reactions ?? {})) {
      if (e) counts.set(e, (counts.get(e) ?? 0) + 1);
    }
    return [...counts.entries()];
  });
</script>

<!-- Double-click anywhere on the row (incoming or outgoing) to reply to it.
     The accessible reply path is the hover Reply button; this is an enhancement. -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="row"
  class:me={message.fromMe}
  class:grouped={sameSenderAsPrev}
  class:highlighted
  data-mid={message.id}
  ondblclick={() => startReply(message)}
  oncontextmenu={onContext}
  use:trackRead={{
    chatJid: message.chatJid,
    id: message.id,
    senderJid: isGroup(message.chatJid) ? message.senderJid : null,
    fromMe: message.fromMe,
  }}
>
  <div class="bubble" class:me={message.fromMe} class:bare={bareSticker}>
    {#if group && !message.fromMe && !sameSenderAsPrev}
      <div class="sender">{senderName}</div>
    {/if}
    {#if message.quoted && !message.deleted}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="quoted" role="button" tabindex="0" onclick={() => scrollToMessage(message.quoted!.id)}
        onkeydown={(e) => (e.key === "Enter" || e.key === " ") && scrollToMessage(message.quoted!.id)}>
        {#if quotedSender}<div class="q-sender">{quotedSender}</div>{/if}
        <div class="q-row">
          <div class="q-text">{message.quoted.text ?? `[${message.quoted.kind}]`}</div>
          {#if message.quoted.thumbnail}
            <img class="q-thumb" src={`data:image/jpeg;base64,${message.quoted.thumbnail}`} alt="" />
          {/if}
        </div>
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
        <div class="text muted">{typeLabel ? `〔${typeLabel}〕` : `[${message.kind}]`}</div>
      {/if}
      {#if typeLabel && message.text && !message.media}
        <div class="type-label">〔{typeLabel}〕</div>
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
        {#if message.fromMe && message.text && !message.media}
          <button class="react-btn" aria-label="Edit" onclick={() => startEdit(message)}>✏</button>
        {/if}
        <div class="react-wrap">
          <button class="react-btn" aria-label="React" onclick={() => (showReact = !showReact)}
            >🙂</button
          >
          {#if showReact}
            <div class="react-pop" class:me={message.fromMe}>
              <ReactionPicker
                onpick={react}
                onclose={() => (showReact = false)}
                current={message.reactions?.[normalizeJid(session.jid ?? "")]}
              />
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
          <span class="tick read">
            <svg viewBox="0 0 100 100" class="tick-svg"
              ><path
                fill="currentColor"
                d="m10 55 20 20 35-45-5-5-30.625 39.375L15 50zm77-25L52 75l-8.687-8.687 4.374-5.626 3.688 3.688L82 25z"
              /></svg
            >
          </span>
        {:else if message.status === "delivered"}
          <span class="tick">
            <svg viewBox="0 0 100 100" class="tick-svg"
              ><path
                fill="currentColor"
                d="m10 55 20 20 35-45-5-5-30.625 39.375L15 50zm77-25L52 75l-8.687-8.687 4.374-5.626 3.688 3.688L82 25z"
              /></svg
            >
          </span>
        {:else}
          <span class="tick">
            <svg viewBox="0 0 100 100" class="tick-svg"
              ><path
                fill="currentColor"
                d="m22.5 52.5 20 20 35-45-5-5-30.625 39.375L27.5 47.5z"
              /></svg
            >
          </span>
        {/if}
      {/if}
    </div>
  </div>
</div>

{#if menu}
  <MessageContextMenu x={menu.x} y={menu.y} {message} onclose={() => (menu = null)} />
{/if}

<style>
  .row {
    display: flex;
    margin-top: 10px;
    border-radius: 6px;
  }
  /* Same-sender follow-on: tight spacing (visually grouped). */
  .row.grouped {
    margin-top: 2px;
  }
  /* Flash when jumped-to from a reply preview. */
  .row.highlighted {
    animation: flashrow 1.4s ease-out;
  }
  @keyframes flashrow {
    0%,
    35% {
      background: color-mix(in srgb, var(--wa-green) 28%, transparent);
    }
    100% {
      background: transparent;
    }
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
    cursor: pointer;
    text-align: left;
    width: 100%;
  }
  .q-sender {
    font-size: 12px;
    font-weight: 600;
    color: var(--wa-green);
  }
  .q-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .q-text {
    flex: 1;
    min-width: 0;
    font-size: 13px;
    color: var(--wa-text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .q-thumb {
    width: 34px;
    height: 34px;
    border-radius: 4px;
    object-fit: cover;
    flex-shrink: 0;
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
  /* Borderless sticker: drop the bubble chrome so the sticker floats. */
  .bubble.bare {
    background: transparent;
    box-shadow: none;
    padding: 0;
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
  .type-label {
    font-size: 11px;
    color: var(--wa-text-muted);
    font-style: italic;
    margin-top: 2px;
  }
  .meta {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 4px;
    font-size: 11px;
    color: var(--wa-text-muted);
    margin-top: 2px;
  }
  .tick {
    display: inline-flex;
    align-items: center;
  }
  /* Tick SVGs read their colour from `color` (fill=currentColor). The vars are
     defined by the theme (app.css); fall back to sensible defaults so this works
     standalone. */
  .tick-svg {
    width: 15px;
    height: 15px;
    display: inline-block;
    color: var(--wa-tick, #8696a0);
  }
  .tick.read .tick-svg {
    color: var(--wa-tick-read, #53bdeb);
  }
</style>
