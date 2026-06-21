<script lang="ts">
  import type { UiMessage } from "../../lib/stores/messages.svelte";
  import { formatTime } from "../../lib/util/time";
  import { jidUser } from "../../lib/util/jid";

  let { message, group }: { message: UiMessage; group: boolean } = $props();
</script>

<div class="row" class:me={message.fromMe}>
  <div class="bubble" class:me={message.fromMe}>
    {#if group && !message.fromMe}
      <div class="sender">{message.pushName ?? jidUser(message.senderJid)}</div>
    {/if}
    {#if message.thumbnail}
      <img class="thumb" src={`data:image/jpeg;base64,${message.thumbnail}`} alt="" />
    {/if}
    {#if message.text}
      <div class="text">{message.text}</div>
    {:else if !message.thumbnail}
      <div class="text muted">[{message.kind}]</div>
    {/if}
    <div class="meta">
      <span>{formatTime(message.timestamp)}</span>
      {#if message.fromMe}<span class="tick">{message.status === "sending" ? "🕓" : "✓"}</span>{/if}
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
    max-width: 65%;
    padding: 6px 9px 4px;
    border-radius: 8px;
    background: var(--wa-bubble-in);
    box-shadow: 0 1px 0.5px #0003;
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
  .meta {
    display: flex;
    justify-content: flex-end;
    gap: 4px;
    font-size: 11px;
    color: var(--wa-text-muted);
    margin-top: 2px;
  }
</style>
