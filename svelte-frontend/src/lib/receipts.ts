// Viewport-driven read receipts: send a "read" ack for an incoming message only
// once it has actually been scrolled into view (like WhatsApp Web), batched and
// debounced. Used as a Svelte action on each incoming message bubble.

import { api } from "./ipc";
import { settings } from "./stores/settings.svelte";

interface Pending {
  chatJid: string;
  senderJid: string | null; // null for DMs; the sender's JID for groups
  id: string;
}

// Already-acknowledged messages (chatJid|id) so we never resend a receipt.
const acked = new Set<string>();
const queue: Pending[] = [];
let timer: ReturnType<typeof setTimeout> | null = null;

let observer: IntersectionObserver | null = null;
const meta = new WeakMap<Element, Pending>();

function ensureObserver(): IntersectionObserver {
  if (observer) return observer;
  observer = new IntersectionObserver(
    (entries) => {
      for (const e of entries) {
        if (!e.isIntersecting) continue;
        const m = meta.get(e.target);
        if (m) enqueue(m);
        observer!.unobserve(e.target); // seen once is enough
      }
    },
    { threshold: 0.6 },
  );
  return observer;
}

function enqueue(p: Pending) {
  if (!settings.sendReadReceipts) return;
  const key = p.chatJid + "|" + p.id;
  if (acked.has(key)) return;
  acked.add(key);
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
    const k = p.chatJid + "||" + (p.senderJid ?? "");
    let g = groups.get(k);
    if (!g) {
      g = { chatJid: p.chatJid, senderJid: p.senderJid, ids: [] };
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
  params: { chatJid: string; id: string; senderJid: string | null; fromMe: boolean },
) {
  if (!params.fromMe && params.id) {
    meta.set(node, { chatJid: params.chatJid, id: params.id, senderJid: params.senderJid });
    ensureObserver().observe(node);
  }
  return {
    destroy() {
      observer?.unobserve(node);
      meta.delete(node);
    },
  };
}
