<script lang="ts">
  import { tick } from "svelte";
  import type { UiMessage } from "../../lib/stores/messages.svelte";
  import { isGroup } from "../../lib/util/jid";
  import { scrollTarget } from "../../lib/stores/scroll.svelte";
  import MessageBubble from "./MessageBubble.svelte";

  let { messages, chatJid }: { messages: UiMessage[]; chatJid: string } = $props();

  // Render only a window of the most recent messages. A full chat can hold
  // thousands of history messages (each with a base64 thumbnail) — mounting
  // them all is what spikes RAM when a chat opens. We start at the latest and
  // reveal older ones as the user scrolls up.
  const PAGE = 40;
  let limit = $state(PAGE);
  let el: HTMLDivElement | undefined = $state();
  // Whether the viewport is parked at the bottom; only then do we auto-follow
  // new messages, so scrolling up to read history doesn't yank you back down.
  let pinned = $state(true);

  const visible = $derived(messages.slice(Math.max(0, messages.length - limit)));
  const hasOlder = $derived(messages.length > limit);

  // Auto-scroll to the newest message on first render and whenever a message
  // arrives while parked at the bottom.
  $effect(() => {
    visible.length; // track
    if (pinned && el) el.scrollTop = el.scrollHeight;
  });

  function jumpToBottom() {
    if (!el) return;
    el.scrollTo({ top: el.scrollHeight, behavior: "smooth" });
    pinned = true;
  }

  // Briefly highlighted message after a jump (tapping a reply's quoted preview).
  let highlightId = $state<string | null>(null);
  let highlightTimer: ReturnType<typeof setTimeout> | null = null;

  // React to a scroll-to-message request: reveal it (growing the window past the
  // loaded page if needed), center it, and flash it.
  $effect(() => {
    scrollTarget.nonce; // track each request
    const id = scrollTarget.id;
    if (id) void revealAndScroll(id);
  });

  async function revealAndScroll(id: string) {
    const idx = messages.findIndex((m) => m.id === id);
    if (idx < 0) return; // not in this chat / not loaded from history
    const fromEnd = messages.length - idx;
    if (fromEnd > limit) {
      limit = fromEnd + 5;
      await tick();
    }
    const node = el?.querySelector(`[data-mid="${CSS.escape(id)}"]`) as HTMLElement | null;
    if (!node) return;
    node.scrollIntoView({ block: "center", behavior: "smooth" });
    highlightId = id;
    if (highlightTimer) clearTimeout(highlightTimer);
    highlightTimer = setTimeout(() => {
      if (highlightId === id) highlightId = null;
    }, 1500);
  }

  async function onScroll() {
    const node = el;
    if (!node) return;
    pinned = node.scrollHeight - node.scrollTop - node.clientHeight < 80;

    // Near the top with more history available → reveal an older page and keep
    // the current messages visually in place (no jump).
    if (node.scrollTop < 60 && hasOlder) {
      const prevHeight = node.scrollHeight;
      const prevTop = node.scrollTop;
      limit += PAGE;
      await tick();
      node.scrollTop = node.scrollHeight - prevHeight + prevTop;
    }
  }
</script>

<div class="wrap">
  <div class="list" bind:this={el} onscroll={onScroll}>
    {#if messages.length === 0}
      <div class="loading">Loading messages…</div>
    {:else}
      {#if hasOlder}
        <div class="older">Showing the latest messages — scroll up for more</div>
      {/if}
      {#each visible as m, i (m.id)}
        <MessageBubble
          message={m}
          group={isGroup(chatJid)}
          prev={visible[i - 1] ?? null}
          highlighted={m.id === highlightId}
        />
      {/each}
    {/if}
  </div>
  {#if !pinned && messages.length > 0}
    <button class="jump" onclick={jumpToBottom} aria-label="Scroll to latest messages">
      <svg viewBox="0 0 24 24" width="20" height="20" aria-hidden="true">
        <path
          d="M12 16.5 4.5 9l1.4-1.4L12 13.7l6.1-6.1L19.5 9z"
          fill="currentColor"
        />
      </svg>
    </button>
  {/if}
</div>

<style>
  .wrap {
    /* Owns the flex sizing so the scroll-to-bottom button can float over the
       list via absolute positioning without consuming scroll space. */
    position: relative;
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  .list {
    flex: 1;
    /* Pair with flex:1 so the list scrolls internally instead of overflowing
       the conversation (which would push the composer off-screen). */
    min-height: 0;
    overflow-y: auto;
    padding: 14px 8%;
    /* Transparent so the conversation wallpaper (set on the parent) shows
       through; the parent defaults to var(--wa-chat-bg) when no wallpaper. */
    background: transparent;
    display: flex;
    flex-direction: column;
  }
  .jump {
    position: absolute;
    right: 18px;
    bottom: 16px;
    width: 42px;
    height: 42px;
    border: none;
    border-radius: 50%;
    background: var(--wa-panel);
    color: var(--wa-text-muted);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.35);
    display: grid;
    place-content: center;
    cursor: pointer;
    z-index: 2;
  }
  .jump:hover {
    background: var(--wa-hover);
    color: var(--wa-text);
  }
  .loading {
    margin: auto;
    color: var(--wa-text-muted);
    font-size: 14px;
  }
  .older {
    align-self: center;
    margin: 4px 0 10px;
    padding: 4px 12px;
    border-radius: 12px;
    background: var(--wa-panel);
    color: var(--wa-text-muted);
    font-size: 12px;
  }
</style>
