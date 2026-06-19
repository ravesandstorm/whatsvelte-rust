<script lang="ts">
  import { api } from "../../lib/ipc";
  import { addOptimistic, confirmOptimistic } from "../../lib/stores/messages.svelte";
  import { touchChat } from "../../lib/stores/chats.svelte";
  import { settings } from "../../lib/stores/settings.svelte";
  import { compose, cancelReply } from "../../lib/stores/compose.svelte";
  import { session } from "../../lib/stores/session.svelte";
  import { normalizeJid, jidUser } from "../../lib/util/jid";
  import { contactFor } from "../../lib/stores/contacts.svelte";
  import type { QuotedDto } from "../../lib/types";
  import EmojiPicker from "./EmojiPicker.svelte";

  let { jid }: { jid: string } = $props();
  let text = $state("");
  let showEmoji = $state(false);
  let ta: HTMLTextAreaElement | undefined = $state();

  const reply = $derived(compose.replyTarget);

  // A reply target belongs to one chat; drop it if we've switched conversations.
  $effect(() => {
    if (compose.replyTarget && compose.replyTarget.chatJid !== jid) cancelReply();
  });

  const replyName = $derived.by(() => {
    const r = compose.replyTarget;
    if (!r) return "";
    if (r.fromMe) return "You";
    if (session.jid && normalizeJid(r.senderJid) === normalizeJid(session.jid)) return "You";
    return contactFor(r.senderJid)?.name ?? jidUser(r.senderJid);
  });

  async function send() {
    const body = text.trim();
    if (!body) return;
    text = "";
    showEmoji = false;
    const target = compose.replyTarget;
    const now = Math.floor(Date.now() / 1000);

    if (target) {
      const quotedSender = target.fromMe ? (session.jid ?? "") : target.senderJid;
      const quoted: QuotedDto = {
        id: target.id,
        senderJid: quotedSender,
        text: target.text,
        kind: target.kind,
      };
      cancelReply();
      const tempId = addOptimistic(jid, body, quoted);
      touchChat(jid, body, now, false);
      try {
        const r = await api.sendReply(jid, body, target.id, quotedSender, target.text);
        confirmOptimistic(jid, tempId, r.messageId);
      } catch (e) {
        console.error("reply failed", e);
      }
      return;
    }

    const tempId = addOptimistic(jid, body);
    touchChat(jid, body, now, false);
    try {
      const r = await api.sendText(jid, body);
      confirmOptimistic(jid, tempId, r.messageId);
    } catch (e) {
      console.error("send failed", e);
    }
  }

  function onKey(e: KeyboardEvent) {
    // Enter-to-send (Shift+Enter = newline) when enabled; otherwise Ctrl/Cmd+
    // Enter sends and a bare Enter inserts a newline.
    const plainEnter = e.key === "Enter" && !e.shiftKey && !e.ctrlKey && !e.metaKey;
    const modEnter = e.key === "Enter" && (e.ctrlKey || e.metaKey);
    if ((settings.enterToSend && plainEnter) || modEnter) {
      e.preventDefault();
      void send();
    }
  }

  function insertEmoji(emoji: string) {
    const el = ta;
    if (!el) {
      text += emoji;
      return;
    }
    const start = el.selectionStart ?? text.length;
    const end = el.selectionEnd ?? text.length;
    text = text.slice(0, start) + emoji + text.slice(end);
    // Restore caret just after the inserted emoji on the next tick.
    queueMicrotask(() => {
      el.focus();
      const pos = start + emoji.length;
      el.setSelectionRange(pos, pos);
    });
  }
</script>

{#if reply}
  <div class="reply-banner">
    <div class="reply-bar"></div>
    <div class="reply-body">
      <div class="reply-name">{replyName}</div>
      <div class="reply-text">{reply.text ?? `[${reply.kind}]`}</div>
    </div>
    <button class="reply-cancel" aria-label="Cancel reply" onclick={cancelReply}>✕</button>
  </div>
{/if}

<div class="composer">
  <div class="emoji-wrap">
    {#if showEmoji}
      <div class="popover">
        <EmojiPicker onpick={insertEmoji} onclose={() => (showEmoji = false)} />
      </div>
    {/if}
    <button
      class="icon"
      aria-label="Emoji"
      class:active={showEmoji}
      onclick={() => (showEmoji = !showEmoji)}>😊</button
    >
  </div>
  <textarea
    bind:this={ta}
    rows="1"
    placeholder="Type a message"
    bind:value={text}
    onkeydown={onKey}
  ></textarea>
  <button class="send" onclick={send} disabled={!text.trim()} aria-label="Send">➤</button>
</div>

<style>
  .composer {
    display: flex;
    gap: 10px;
    align-items: center;
    padding: 10px 16px;
    background: var(--wa-panel);
  }
  .reply-banner {
    display: flex;
    align-items: stretch;
    gap: 8px;
    margin: 0 16px;
    padding: 6px 8px;
    background: var(--wa-panel-2);
    border-radius: 6px 6px 0 0;
  }
  .reply-bar {
    width: 3px;
    border-radius: 2px;
    background: var(--wa-green);
    flex-shrink: 0;
  }
  .reply-body {
    flex: 1;
    min-width: 0;
  }
  .reply-name {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--wa-green);
  }
  .reply-text {
    font-size: 13px;
    color: var(--wa-text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .reply-cancel {
    border: none;
    background: transparent;
    color: var(--wa-text-muted);
    font-size: 14px;
    align-self: flex-start;
  }
  .emoji-wrap {
    position: relative;
    flex-shrink: 0;
  }
  .popover {
    position: absolute;
    bottom: calc(100% + 10px);
    left: 0;
    z-index: 20;
  }
  .icon {
    width: 40px;
    height: 40px;
    border: none;
    border-radius: 50%;
    background: transparent;
    font-size: 20px;
    color: var(--wa-text-muted);
  }
  .icon:hover,
  .icon.active {
    background: var(--wa-panel-2);
    color: var(--wa-text);
  }
  textarea {
    flex: 1;
    resize: none;
    max-height: 120px;
    padding: 10px 14px;
    border: none;
    border-radius: 8px;
    background: var(--wa-panel-2);
    color: var(--wa-text);
    line-height: 1.3;
  }
  textarea:focus {
    outline: none;
  }
  .send {
    width: 42px;
    height: 42px;
    border: none;
    border-radius: 50%;
    background: var(--wa-green);
    color: #04221c;
    font-size: 16px;
    flex-shrink: 0;
  }
  .send:disabled {
    opacity: 0.4;
  }
</style>
