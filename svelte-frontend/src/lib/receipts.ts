// Viewport-driven read receipts: send a "read" ack for an incoming message only
// once it has actually been scrolled into view (like WhatsApp Web), batched and
// debounced. Used as a Svelte action on each incoming message bubble.

import { api } from "./ipc";
import { settings } from "./stores/settings.svelte";
import { markMessageRead } from "./stores/messages.svelte";

interface Pending {
  chatJid: string; // canonical key (local state)
  wireChatJid: string; // as the server addressed it (the receipt goes here)
  senderJid: string | null; // null for DMs; the sender's wire JID for groups
  id: string;
}

// Already-acknowledged messages (chatJid|id) so we never resend a receipt.
// Bounded (oldest dropped first): a long session would otherwise grow it by one
// entry per message ever read. A dropped entry only risks a duplicate receipt.
const acked = new Set<string>();
const ACKED_MAX = 5000;
const queue: Pending[] = [];
let timer: ReturnType<typeof setTimeout> | null = null;

let observer: IntersectionObserver | null = null;
const meta = new WeakMap<Element, Pending[]>();

function ensureObserver(): IntersectionObserver {
  if (observer) return observer;
  observer = new IntersectionObserver(
    (entries) => {
      for (const e of entries) {
        if (!e.isIntersecting) continue;
        for (const m of meta.get(e.target) ?? []) enqueue(m);
        observer!.unobserve(e.target); // seen once is enough
      }
    },
    { threshold: 0.6 },
  );
  return observer;
}

function enqueue(p: Pending) {
  markMessageRead(p.chatJid, p.id);
  if (!settings.sendReadReceipts) return;
  const key = p.chatJid + "|" + p.id;
  if (acked.has(key)) return;
  acked.add(key);
  if (acked.size > ACKED_MAX) acked.delete(acked.values().next().value!);
  queue.push(p);
  if (!timer) timer = setTimeout(flush, 600);
}

async function flush() {
  timer = null;
  const batch = queue.splice(0);
  if (!batch.length) return;
  // The library marks per (chat, sender), so group accordingly.
  const groups = new Map<string, { chatJid: string; senderJid: string | null; ids: string[] }>();
  for (const p of batch) {
    const k = p.wireChatJid + "||" + (p.senderJid ?? "");
    let g = groups.get(k);
    if (!g) {
      g = { chatJid: p.wireChatJid, senderJid: p.senderJid, ids: [] };
      groups.set(k, g);
    }
    g.ids.push(p.id);
  }
  for (const g of groups.values()) {
    try {
      await api.markReadMessages(g.chatJid, g.senderJid, g.ids);
    } catch (e) {
      console.error("mark read failed", e);
    }
  }
}

/** Svelte action: ack an incoming bubble once it scrolls into view. */
export function trackRead(
  node: HTMLElement,
  params: {
    chatJid: string;
    wireChatJid: string;
    id: string;
    /** Several messages shown by one element (an album grid). */
    ids?: string[];
    senderJid: string | null;
    fromMe: boolean;
  },
) {
  if (!params.fromMe && params.id) {
    const ids = params.ids?.length ? params.ids : [params.id];
    meta.set(
      node,
      ids.map((id) => ({
        chatJid: params.chatJid,
        wireChatJid: params.wireChatJid,
        id,
        senderJid: params.senderJid,
      })),
    );
    ensureObserver().observe(node);
  }
  return {
    destroy() {
      observer?.unobserve(node);
      meta.delete(node);
    },
  };
}
