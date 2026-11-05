<script lang="ts">
  import { tick, untrack } from "svelte";
  import {
    closeChat,
    hasOlder as olderInDb,
    loadOlder,
    loadUntil,
    markMessagesRead,
    openChat,
    trimChat,
    type UiMessage,
  } from "../../lib/stores/messages.svelte";
  import { isGroup } from "../../lib/util/jid";
  import { scrollTarget, clearScrollTarget } from "../../lib/stores/scroll.svelte";
  import MessageBubble from "./MessageBubble.svelte";

  let { messages, chatJid }: { messages: UiMessage[]; chatJid: string } = $props();

  // Render only a window of the most recent messages. A full chat can hold
  // thousands of history messages (each with a base64 thumbnail) — mounting
  // them all is what spikes RAM when a chat opens. We start at the latest and
  // reveal older ones as the user scrolls up. The store itself only holds a
  // short tail per chat; older pages are read from IndexedDB on demand.
  const PAGE = 40;
  let limit = $state(PAGE);
  let el: HTMLDivElement | undefined = $state();
  // Whether the viewport is parked at the bottom; only then do we auto-follow
  // new messages, so scrolling up to read history doesn't yank you back down.
  let pinned = $state(true);

  const visible = $derived(messages.slice(Math.max(0, messages.length - limit)));
  const hasOlder = $derived(messages.length > limit || olderInDb(chatJid));

  // Identify the first unread incoming message to render the "Unread" divider above it
  const firstUnreadId = $derived.by(() => {
    const first = visible.find((m) => !m.fromMe && m.unread);
    return first ? first.id : null;
  });

  const unreadCount = $derived(visible.filter((m) => !m.fromMe && m.unread).length);

  let prevJid = $state<string | null>(null);
  let prevLastId = $state<string | null>(null);

  // Page the chat's history in from IndexedDB while it's open, and drop it back
  // to the resident tail when it's closed (switch or unmount).
  $effect(() => {
    const jid = chatJid;
    return () => closeChat(jid);
  });

  // Initial scroll positioning on chat switch or mount
  $effect(() => {
    if (chatJid !== prevJid) {
      const jid = chatJid;
      prevJid = jid;
      limit = PAGE;
      untrack(() => {
        prevLastId = visible[visible.length - 1]?.id ?? null;
      });
      void untrack(() => openChat(jid)).then(async () => {
        if (jid !== chatJid) return; // switched away while loading
        prevLastId = untrack(() => visible[visible.length - 1]?.id ?? null);
        await tick();
        if (!el) return;
        const unreadMsg = visible.find((m) => !m.fromMe && m.unread);
        if (unreadMsg) {
          const divider = el.querySelector("[data-unread-divider]") as HTMLElement | null;
          if (divider) {
            divider.scrollIntoView({ block: "start" });
            pinned = false;
          } else {
            el.scrollTop = el.scrollHeight;
            pinned = true;
          }
        } else {
          el.scrollTop = el.scrollHeight;
          pinned = true;
        }
      });
    }
  });

  // Handle auto-scroll logic on new messages:
  // Outgoing messages auto-scroll to bottom.
  // Incoming messages stay unread below the scroll point without auto-scrolling down!
  // Keyed on the newest message's id (not the list length), so older pages
  // being prepended from IndexedDB never count as a new arrival.
  $effect(() => {
    const newest = visible[visible.length - 1];
    const lastId = newest?.id ?? null;
    if (chatJid !== prevJid || lastId === prevLastId) return;
    const isNewArrival = prevLastId !== null;
    prevLastId = lastId;
    if (!isNewArrival || !newest) return;
    if (newest.fromMe) {
      void tick().then(() => {
        if (el) el.scrollTop = el.scrollHeight;
        pinned = true;
      });
    } else {
      pinned = false;
    }
  });

  function jumpToBottom() {
    if (!el) return;
    el.scrollTo({ top: el.scrollHeight, behavior: "smooth" });
    pinned = true;
    const unreadIds = visible.filter((m) => !m.fromMe && m.unread).map((m) => m.id);
    if (unreadIds.length > 0) {
      markMessagesRead(chatJid, unreadIds);
    }
  }

  // Briefly highlighted message after a jump (tapping a reply's quoted preview).
  let highlightId = $state<string | null>(null);
  let highlightTimer: ReturnType<typeof setTimeout> | null = null;

  // React to a scroll-to-message request: reveal it (growing the window past the
  // loaded page if needed), center it, and flash it.
  $effect(() => {
    scrollTarget.nonce; // sole trigger — fires exactly once per explicit request
    // Read `id` untracked so clearing it later (clearScrollTarget) can't re-fire
    // this effect and yank the viewport back to the message after the user has
    // scrolled away.
    const id = untrack(() => scrollTarget.id);
    if (id) void revealAndScroll(id);
  });

  async function revealAndScroll(id: string) {
    // Page older history in from IndexedDB until the target is resident.
    if (!messages.some((m) => m.id === id) && !(await loadUntil(chatJid, id))) return;
    await tick();
    const idx = messages.findIndex((m) => m.id === id);
    if (idx < 0) return; // not in this chat / never persisted
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
      clearScrollTarget(); // disarm the request now the flash is done
    }, 1500);
  }

  let pagingOlder = false;

  async function onScroll() {
    const node = el;
    if (!node) return;
    const isAtBottom = node.scrollHeight - node.scrollTop - node.clientHeight < 40;
    pinned = isAtBottom;

    // Back at the latest messages after reading far up: unmount the old pages
    // and release them from memory (they reload from IndexedDB on scroll-up),
    // so a long scroll session doesn't keep every image and bubble alive.
    if (isAtBottom && limit > PAGE * 2) {
      limit = PAGE;
      trimChat(chatJid, PAGE);
      await tick();
      node.scrollTop = node.scrollHeight;
    }

    if (isAtBottom && unreadCount > 0) {
      const unreadIds = visible.filter((m) => !m.fromMe && m.unread).map((m) => m.id);
      if (unreadIds.length > 0) {
        markMessagesRead(chatJid, unreadIds);
      }
    }

    // Near the top with more history available → reveal an older page and keep
    // the current messages visually in place (no jump).
    if (node.scrollTop < 60 && hasOlder && !pagingOlder) {
      pagingOlder = true;
      const prevHeight = node.scrollHeight;
      const prevTop = node.scrollTop;
      try {
        // Out of resident messages → read the next page from IndexedDB first.
        if (messages.length <= limit) await loadOlder(chatJid, PAGE);
      } finally {
        pagingOlder = false;
      }
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
        {#if m.id === firstUnreadId}
          <div class="unread-divider" data-unread-divider>
            <span>Unread</span>
          </div>
        {/if}
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
      {#if unreadCount > 0}
        <span class="jump-badge">{unreadCount}</span>
      {/if}
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
  .unread-divider {
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 14px 0;
    position: relative;
  }
  .unread-divider::before {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    top: 50%;
    height: 1px;
    background: var(--wa-border, rgba(255, 255, 255, 0.15));
    z-index: 1;
  }
  .unread-divider span {
    position: relative;
    z-index: 2;
    background: var(--wa-panel, #202c33);
    color: var(--wa-unread, #00a884);
    font-size: 12px;
    font-weight: 500;
    padding: 4px 14px;
    border-radius: 12px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.25);
    text-transform: uppercase;
    letter-spacing: 0.5px;
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
  .jump-badge {
    position: absolute;
    top: -4px;
    right: -2px;
    background: var(--wa-unread, #00a884);
    color: #ffffff;
    font-size: 11px;
    font-weight: 600;
    height: 18px;
    min-width: 18px;
    padding: 0 5px;
    border-radius: 9px;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 2px 5px rgba(0, 0, 0, 0.3);
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
