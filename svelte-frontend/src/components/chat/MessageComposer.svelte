<script lang="ts">
  import { api } from "../../lib/ipc";
  import { addOptimistic, applyEdit, confirmOptimistic } from "../../lib/stores/messages.svelte";
  import { touchChat } from "../../lib/stores/chats.svelte";
  import { settings } from "../../lib/stores/settings.svelte";
  import { compose, cancelReply, cancelEdit } from "../../lib/stores/compose.svelte";
  import { session } from "../../lib/stores/session.svelte";
  import { normalizeJid, formatPhone } from "../../lib/util/jid";
  import { contactFor } from "../../lib/stores/contacts.svelte";
  import type { QuotedDto } from "../../lib/types";
  import { searchEmojiShortcodes } from "../../lib/emoji";
  import EmojiPicker from "./EmojiPicker.svelte";
  import AttachMenu from "./AttachMenu.svelte";
  import MediaPreview from "./MediaPreview.svelte";
  import Recorder from "./Recorder.svelte";
  import type { PendingAttachment } from "../../lib/send-media";

  let { jid }: { jid: string } = $props();
  let text = $state("");
  let showEmoji = $state(false);
  let pending = $state<PendingAttachment | null>(null);
  let recordMode = $state<"voice" | "video" | "photo" | null>(null);
  let ta: HTMLTextAreaElement | undefined = $state();
  let lastEditId: string | null = $state(null);
  // `:shortcode` autocomplete state.
  let suggest = $state<{ code: string; emoji: string }[]>([]);
  let suggestIdx = $state(0);

  const TOKEN_RE = /(?:^|\s):([a-zA-Z0-9_+-]+)$/;

  // Recompute the `:`-suggestion list from the text just before the caret.
  function updateSuggest() {
    const el = ta;
    if (!el) return;
    const before = text.slice(0, el.selectionStart ?? text.length);
    const m = before.match(TOKEN_RE);
    suggest = m ? searchEmojiShortcodes(m[1]) : [];
    suggestIdx = 0;
  }

  function acceptSuggest(item: { code: string; emoji: string }) {
    const el = ta;
    const caret = el?.selectionStart ?? text.length;
    const before = text.slice(0, caret);
    const m = before.match(/:([a-zA-Z0-9_+-]+)$/);
    if (!m) return;
    const start = caret - m[0].length;
    text = text.slice(0, start) + item.emoji + text.slice(caret);
    suggest = [];
    queueMicrotask(() => {
      el?.focus();
      const pos = start + item.emoji.length;
      el?.setSelectionRange(pos, pos);
    });
  }

  const reply = $derived(compose.replyTarget);
  const editing = $derived(compose.editTarget);

  // A reply/edit target belongs to one chat; drop it if we've switched chats.
  $effect(() => {
    if (compose.replyTarget && compose.replyTarget.chatJid !== jid) cancelReply();
    if (compose.editTarget && compose.editTarget.chatJid !== jid) cancelEdit();
  });

  // Prefill the box with the message being edited (once per new edit target).
  $effect(() => {
    const e = compose.editTarget;
    if (e && e.chatJid === jid && e.id !== lastEditId) {
      text = e.text ?? "";
      lastEditId = e.id;
      queueMicrotask(() => ta?.focus());
    } else if (!e) {
      lastEditId = null;
    }
  });

  const replyName = $derived.by(() => {
    const r = compose.replyTarget;
    if (!r) return "";
    if (r.fromMe) return "You";
    if (session.jid && normalizeJid(r.senderJid) === normalizeJid(session.jid)) return "You";
    return contactFor(r.senderJid)?.name ?? formatPhone(r.senderJid);
  });

  async function send() {
    const body = text.trim();
    if (!body) return;
    showEmoji = false;
    suggest = [];
    const now = Math.floor(Date.now() / 1000);

    // Edit takes priority over reply/new-message.
    const edit = compose.editTarget;
    if (edit) {
      text = "";
      cancelEdit();
      applyEdit(edit.chatJid, edit.id, body, now); // optimistic
      try {
        await api.editMessage(jid, edit.id, body);
      } catch (e) {
        console.error("edit failed", e);
      }
      return;
    }

    text = "";
    const target = compose.replyTarget;

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
    // When the suggestion popover is open, arrows/enter/tab drive it.
    if (suggest.length) {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        suggestIdx = (suggestIdx + 1) % suggest.length;
        return;
      }
      if (e.key === "ArrowUp") {
        e.preventDefault();
        suggestIdx = (suggestIdx - 1 + suggest.length) % suggest.length;
        return;
      }
      if (e.key === "Enter" || e.key === "Tab") {
        e.preventDefault();
        acceptSuggest(suggest[suggestIdx]);
        return;
      }
      if (e.key === "Escape") {
        e.preventDefault();
        suggest = [];
        return;
      }
    }
    // Enter-to-send (Shift+Enter = newline) when enabled; otherwise Ctrl/Cmd+
    // Enter sends and a bare Enter inserts a newline.
    const plainEnter = e.key === "Enter" && !e.shiftKey && !e.ctrlKey && !e.metaKey;
    const modEnter = e.key === "Enter" && (e.ctrlKey || e.metaKey);
    if ((settings.enterToSend && plainEnter) || modEnter) {
      e.preventDefault();
      void send();
    }
  }

  // Clipboard images → the same pre-send preview as the attach menu.
  function onPaste(e: ClipboardEvent) {
    const items = e.clipboardData?.items;
    if (!items) return;
    for (const it of items) {
      if (it.kind === "file" && it.type.startsWith("image/")) {
        const file = it.getAsFile();
        if (file) {
          e.preventDefault();
          pending = {
            kind: "image",
            blob: file,
            fileName: file.name || "pasted-image.png",
            previewUrl: URL.createObjectURL(file),
          };
          return;
        }
      }
    }
  }

  function closePreview() {
    if (pending?.previewUrl) URL.revokeObjectURL(pending.previewUrl);
    pending = null;
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

{#if editing}
  <div class="reply-banner">
    <div class="reply-bar"></div>
    <div class="reply-body">
      <div class="reply-name">Editing message</div>
      <div class="reply-text">{editing.text ?? `[${editing.kind}]`}</div>
    </div>
    <button class="reply-cancel" aria-label="Cancel edit" onclick={cancelEdit}>✕</button>
  </div>
{:else if reply}
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
      <EmojiPicker overlay onpick={insertEmoji} onclose={() => (showEmoji = false)} />
    {/if}
    <button
      class="icon"
      aria-label="Emoji"
      class:active={showEmoji}
      onclick={() => (showEmoji = !showEmoji)}>😊</button
    >
  </div>
  <AttachMenu onpick={(a) => (pending = a)} onrecord={(m) => (recordMode = m)} />
  <div class="ta-wrap">
    {#if suggest.length}
      <div class="suggest">
        {#each suggest as s, i (s.code)}
          <button
            class="suggest-item"
            class:active={i === suggestIdx}
            onclick={() => acceptSuggest(s)}
          >
            <span class="suggest-emoji">{s.emoji}</span>
            <span class="suggest-code">:{s.code}:</span>
          </button>
        {/each}
      </div>
    {/if}
    <textarea
      bind:this={ta}
      rows="1"
      placeholder="Type a message"
      bind:value={text}
      onkeydown={onKey}
      oninput={updateSuggest}
      onclick={updateSuggest}
      onpaste={onPaste}
    ></textarea>
  </div>
  <button class="send" onclick={send} disabled={!text.trim()} aria-label="Send">➤</button>
</div>

{#if recordMode}
  <Recorder
    mode={recordMode}
    onclose={() => (recordMode = null)}
    oncapture={(a) => {
      recordMode = null;
      pending = a;
    }}
  />
{/if}

{#if pending}
  <MediaPreview {jid} attachment={pending} onclose={closePreview} />
{/if}

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
  .ta-wrap {
    flex: 1;
    position: relative;
    display: flex;
  }
  .suggest {
    position: absolute;
    bottom: calc(100% + 8px);
    left: 0;
    z-index: 1000;
    min-width: 200px;
    max-height: 220px;
    overflow-y: auto;
    background: var(--wa-panel-2);
    border: 1px solid var(--wa-border);
    border-radius: 8px;
    padding: 4px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  }
  .suggest-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    border: none;
    background: transparent;
    color: var(--wa-text);
    padding: 6px 8px;
    border-radius: 6px;
    text-align: left;
  }
  .suggest-item.active,
  .suggest-item:hover {
    background: var(--wa-hover);
  }
  .suggest-emoji {
    font-size: 20px;
  }
  .suggest-code {
    font-size: 13px;
    color: var(--wa-text-muted);
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
