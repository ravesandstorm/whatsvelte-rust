// Mock backend for frontend-only work: no Tauri, no WhatsApp connection.
//
// Enable with `npm run dev:mock` (VITE_MOCK=1) or by opening the dev server with
// `?mock` in the URL. It stubs every Tauri command via mockIPC and feeds the
// normal wa:// events, so the real hydration/stores/persistence code runs
// unchanged. IndexedDB is wiped on each load so the dataset is deterministic.
//
// Tunables (query string): ?mock&chats=120&msgs=400  (defaults 80 / 300)
// ?timerraf: timer-based requestAnimationFrame for hidden/headless webviews.
// Console helpers: window.__mock.incoming(jid?, text?) · .history(jid, n)

import { mockIPC } from "@tauri-apps/api/mocks";
import { emit } from "@tauri-apps/api/event";
import type { ChatDto, MessageDto } from "./types";

export function mockEnabled(): boolean {
  if (import.meta.env.VITE_MOCK === "1") return true;
  return typeof location !== "undefined" && new URLSearchParams(location.search).has("mock");
}

const ME = "919999900000@s.whatsapp.net";
const NOW = Math.floor(Date.now() / 1000);

const NAMES = [
  "Aarav", "Diya", "Kabir", "Meera", "Rohan", "Isha", "Vihaan", "Anaya", "Arjun", "Sara",
  "Neil", "Tara", "Dev", "Zoya", "Kian", "Riya", "Om", "Nisha", "Yash", "Pooja",
];
const LINES = [
  "Hey, are you free later?",
  "Sent you the doc, take a look",
  "lol that's hilarious",
  "Can we move the call to 4?",
  "Thanks! 🙏",
  "Running 10 mins late",
  "Did you see the news today? It's honestly wild how fast things are changing.",
  "ok",
  "Let me check and get back to you",
  "Sounds good to me 👍",
  "Which one did you pick in the end? I couldn't decide between the two.",
  "Happy birthday!! 🎉🎂",
];

// Small deterministic PRNG so every run produces the same data.
let seed = 42;
const rnd = () => ((seed = (seed * 1664525 + 1013904223) >>> 0) / 4294967296);
const pick = <T,>(a: T[]): T => a[Math.floor(rnd() * a.length)];

const q = typeof location !== "undefined" ? new URLSearchParams(location.search) : new URLSearchParams();
const CHAT_COUNT = Number(q.get("chats")) || 80;
const MSGS_PER_CHAT = Number(q.get("msgs")) || 300;

const chatJid = (i: number) => (i % 9 === 8 ? `1203630${i}@g.us` : `9198000${String(i).padStart(5, "0")}@s.whatsapp.net`);
const isGroup = (jid: string) => jid.endsWith("@g.us");

let idCounter = 0;
function makeMessage(jid: string, ts: number, fromMe: boolean, text?: string): MessageDto {
  const sender = fromMe ? ME : isGroup(jid) ? `9198111${String(Math.floor(rnd() * 5)).padStart(5, "0")}@s.whatsapp.net` : jid;
  return {
    id: `MOCK${(idCounter++).toString(36).toUpperCase().padStart(8, "0")}`,
    chatJid: jid,
    senderJid: sender,
    fromMe,
    timestamp: ts,
    pushName: fromMe ? null : pick(NAMES),
    text: text ?? pick(LINES),
    kind: "text",
    thumbnail: null,
    media: null,
    quoted: null,
  };
}

function buildChat(i: number): { chat: ChatDto; messages: MessageDto[] } {
  const jid = chatJid(i);
  const n = i === 0 ? Math.max(MSGS_PER_CHAT, 600) : Math.max(5, Math.floor(rnd() * MSGS_PER_CHAT));
  const start = NOW - i * 3600 - n * 90;
  const messages: MessageDto[] = [];
  for (let k = 0; k < n; k++) messages.push(makeMessage(jid, start + k * 90, rnd() < 0.4));
  const last = messages[messages.length - 1];
  return {
    chat: {
      jid,
      name: isGroup(jid) ? `Group ${i}` : `${NAMES[i % NAMES.length]} ${i}`,
      lastMessage: last.text,
      timestamp: last.timestamp,
      unread: i % 5 === 1 ? 3 : 0,
    },
    messages,
  };
}

const data = Array.from({ length: CHAT_COUNT }, (_, i) => buildChat(i));

const env = <T,>(kind: string, payload: T) => ({ sessionId: "default", kind, payload });
const send = <T,>(topic: string, payload: T) => emit(topic, env(topic, payload));

async function wipeIndexedDb(): Promise<void> {
  if (typeof indexedDB === "undefined") return;
  await new Promise<void>((res) => {
    const r = indexedDB.deleteDatabase("whatsvelte");
    r.onsuccess = r.onerror = r.onblocked = () => res();
  });
}

export async function installMock(): Promise<void> {
  // Headless/hidden webviews never fire requestAnimationFrame. `?timerraf`
  // swaps in a timer-based one so the app (and its tests) still run there.
  if (q.has("timerraf")) {
    window.requestAnimationFrame = (cb) => window.setTimeout(() => cb(performance.now()), 16);
    window.cancelAnimationFrame = (id) => window.clearTimeout(id);
  }
  await wipeIndexedDb();

  mockIPC(
    (cmd, args) => {
      const a = (args ?? {}) as Record<string, any>;
      switch (cmd) {
        case "auth_status":
          return {
            sessionId: "default", loggedIn: true, connected: true,
            jid: ME, lid: null, registered: true, pushName: "Me (mock)",
          };
        case "list_newsletters":
          return [];
        case "send_text":
        case "send_reply":
          return { sessionId: "default", messageId: `MOCKSENT${Date.now().toString(36)}`, to: a.jid };
        case "get_contact":
          return { jid: a.jid, name: null, verifiedName: null, lid: null, pictureUrl: null };
        case "resolve_jid":
          return { pn: a.jid, lid: null };
        case "get_profile_picture_url":
          return null;
        default:
          return null; // mark_read, mute/pin/archive, reactions, etc.: no-ops
      }
    },
    { shouldMockEvents: true },
  );

  // Deliver the history once the event bridge has registered its listeners
  // (it does that before reading the cache, so a short delay is enough).
  setTimeout(() => {
    void send("wa://history", {
      chats: data.map((d) => d.chat),
      messages: data.flatMap((d) => d.messages),
    });
  }, 400);

  (window as any).__mock = {
    /** Simulate a live incoming message (bottom update). */
    incoming(jid = data[0].chat.jid, text = "Mock incoming message") {
      void send("wa://message", makeMessage(jid, Math.floor(Date.now() / 1000), false, text));
    },
    /** Simulate a history-sync chunk of older messages (top update). */
    history(jid = data[0].chat.jid, n = 40) {
      const oldest = data.find((d) => d.chat.jid === jid)?.messages[0]?.timestamp ?? NOW;
      const msgs = Array.from({ length: n }, (_, k) => makeMessage(jid, oldest - (n - k) * 90, false));
      void send("wa://history", { chats: [], messages: msgs });
    },
  };
}
