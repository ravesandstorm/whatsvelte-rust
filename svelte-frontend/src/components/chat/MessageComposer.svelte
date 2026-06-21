<script lang="ts">
  import { api } from "../../lib/ipc";
  import { addOptimistic, confirmOptimistic } from "../../lib/stores/messages.svelte";
  import { touchChat } from "../../lib/stores/chats.svelte";

  let { jid }: { jid: string } = $props();
  let text = $state("");

  async function send() {
    const body = text.trim();
    if (!body) return;
    text = "";
    const tempId = addOptimistic(jid, body);
    touchChat(jid, body, Math.floor(Date.now() / 1000), false);
    try {
      const r = await api.sendText(jid, body);
      confirmOptimistic(jid, tempId, r.messageId);
    } catch (e) {
      console.error("send failed", e);
    }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      void send();
    }
  }
</script>

<div class="composer">
  <textarea
    rows="1"
    placeholder="Type a message"
    bind:value={text}
    onkeydown={onKey}
  ></textarea>
  <button onclick={send} disabled={!text.trim()} aria-label="Send">➤</button>
</div>

<style>
  .composer {
    display: flex;
    gap: 10px;
    align-items: center;
    padding: 10px 16px;
    background: var(--wa-panel);
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
  button {
    width: 42px;
    height: 42px;
    border: none;
    border-radius: 50%;
    background: var(--wa-green);
    color: #04221c;
    font-size: 16px;
    flex-shrink: 0;
  }
  button:disabled {
    opacity: 0.4;
  }
</style>
