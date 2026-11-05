// IndexedDB persistence + bounded in-memory message store.
//
// Each test gets a fresh fake IndexedDB and freshly imported modules, because
// persist.ts / the stores keep module-level state (db handle, write queue,
// reactive maps).

import "fake-indexeddb/auto"; // installs IDBKeyRange etc. as globals
import { IDBFactory } from "fake-indexeddb";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { UiMessage } from "./stores/messages.svelte";

type Persist = typeof import("./persist");
type Msgs = typeof import("./stores/messages.svelte");
type Chats = typeof import("./stores/chats.svelte");

let persist: Persist;
let msgs: Msgs;
let chats: Chats;

const A = "a@s.whatsapp.net";
const B = "b@s.whatsapp.net";
const C = "c@s.whatsapp.net";
const E = "e@s.whatsapp.net";
const G = "g@g.us";

function mk(chatJid: string, i: number, extra: Partial<UiMessage> = {}): UiMessage {
  return {
    id: "m" + String(i).padStart(4, "0"),
    chatJid,
    senderJid: chatJid,
    fromMe: false,
    timestamp: 1000 + i,
    pushName: "P",
    text: "t" + i,
    kind: "text",
    thumbnail: null,
    media: null,
    quoted: null,
    ...extra,
  };
}

async function load() {
  vi.resetModules();
  persist = await import("./persist");
  msgs = await import("./stores/messages.svelte");
  chats = await import("./stores/chats.svelte");
}

/** Write `items` straight to the store (bypassing the debounce). */
async function seed(items: UiMessage[]) {
  persist.enablePersistence();
  items.forEach(persist.persistMessage);
  await persist.flushPendingWrites();
}

beforeEach(async () => {
  globalThis.indexedDB = new IDBFactory();
  await load();
});

describe("v1 → v2 migration", () => {
  beforeEach(async () => {
    // Seed a v1 database (one row per chat) before persist.ts opens it.
    await new Promise<void>((res, rej) => {
      const r = indexedDB.open("whatsvelte", 1);
      r.onupgradeneeded = () => {
        const db = r.result;
        db.createObjectStore("chats", { keyPath: "jid" });
        db.createObjectStore("messages", { keyPath: "jid" });
        db.createObjectStore("meta", { keyPath: "k" });
      };
      r.onsuccess = () => {
        const db = r.result;
        const t = db.transaction(["chats", "messages", "meta"], "readwrite");
        const itemsA = Array.from({ length: 60 }, (_, i) => mk(A, i));
        itemsA.push(mk(A, 999, { id: "tmp-x", status: "sending" }));
        t.objectStore("messages").put({ jid: A, items: itemsA });
        t.objectStore("messages").put({ jid: B, items: Array.from({ length: 5 }, (_, i) => mk(B, i)) });
        t.objectStore("chats").put({ jid: B, name: "Bee", lastMessage: "t4", timestamp: 1004, unread: 0 });
        t.objectStore("meta").put({ k: "accountJid", v: "me@s.whatsapp.net" });
        t.oncomplete = () => {
          db.close();
          res();
        };
        t.onerror = () => rej(t.error);
      };
    });
  });

  it("creates missing chat rows and keeps the account", async () => {
    const snap = await persist.loadSnapshot();
    expect(snap.chats.map((c) => c.jid).sort()).toEqual([A, B]);
    expect(snap.hasMessages).toBe(true);
    expect(snap.accountJid).toBe("me@s.whatsapp.net");
  });

  it("migrates persistable messages oldest-first and skips optimistic ones", async () => {
    const all = await persist.loadLatestMessages(A, 1000);
    expect(all).toHaveLength(60);
    expect(all[0].id).toBe("m0000");
    expect(all[59].id).toBe("m0059");
    expect(all.some((m) => m.id.startsWith("tmp-"))).toBe(false);
  });
});

describe("v2 → v3: blank [other] rows", () => {
  beforeEach(async () => {
    // A v2 database holding protocol traffic stored as blank "other" rows.
    await new Promise<void>((res, rej) => {
      const r = indexedDB.open("whatsvelte", 2);
      r.onupgradeneeded = () => {
        const db = r.result;
        db.createObjectStore("chats", { keyPath: "jid" });
        db.createObjectStore("meta", { keyPath: "k" });
        const msgs = db.createObjectStore("msgs", { keyPath: ["chatJid", "id"] });
        msgs.createIndex("byChatTime", ["chatJid", "timestamp"]);
      };
      r.onsuccess = () => {
        const db = r.result;
        const t = db.transaction(["msgs"], "readwrite");
        const s = t.objectStore("msgs");
        s.put(mk(A, 1));
        s.put(mk(A, 2, { kind: "other", text: null }));
        s.put(mk(A, 3, { kind: "other", text: null, deleted: true }));
        s.put(mk(A, 4, { kind: "other", text: "readable" }));
        t.oncomplete = () => {
          db.close();
          res();
        };
        t.onerror = () => rej(t.error);
      };
    });
  });

  it("drops rows with nothing to render, keeps tombstones and text", async () => {
    const ids = (await persist.loadLatestMessages(A, 100)).map((m) => m.id);
    expect(ids).toEqual(["m0001", "m0003", "m0004"]);
  });

  it("never stores or shows a new blank message", async () => {
    persist.enablePersistence();
    msgs.addMessage(mk(C, 9, { kind: "other", text: null }));
    await persist.flushPendingWrites();
    expect(msgs.messagesFor(C)).toHaveLength(0);
    expect(await persist.loadLatestMessages(C, 10)).toHaveLength(0);
  });
});

describe("boot and paging", () => {
  beforeEach(async () => {
    await seed(Array.from({ length: 60 }, (_, i) => mk(A, i)));
    chats.hydrateChat({ jid: A, name: null, lastMessage: null, timestamp: 1059, unread: 0 });
    const tails = await persist.loadTails([A], 1);
    msgs.addHistoryMessages([...tails.values()].flat(), { persist: false });
  });

  it("boot keeps only the newest message per chat", () => {
    expect(msgs.messagesFor(A).map((m) => m.id)).toEqual(["m0059"]);
  });

  it("opening a chat tops it up to one page, then pages to the start", async () => {
    chats.selectChat(A);
    await msgs.openChat(A);
    expect(msgs.messagesFor(A)).toHaveLength(msgs.PAGE);
    expect(msgs.hasOlder(A)).toBe(true);

    await msgs.loadOlder(A);
    expect(msgs.messagesFor(A)).toHaveLength(60);
    expect(msgs.hasOlder(A)).toBe(false);
  });

  it("closing a chat trims it back to the resident tail", async () => {
    chats.selectChat(A);
    await msgs.openChat(A);
    chats.selectChat(B);
    msgs.closeChat(A);
    expect(msgs.messagesFor(A)).toHaveLength(msgs.RESIDENT_TAIL);
    expect(msgs.messagesFor(A).at(-1)?.id).toBe("m0059");
    expect(msgs.hasOlder(A)).toBe(true);
  });

  it("pages through many messages sharing one timestamp", async () => {
    await seed(Array.from({ length: 120 }, (_, i) => mk(G, i, { timestamp: 777 })));
    chats.selectChat(G);
    for (let i = 0; i < 20 && msgs.hasOlder(G); i++) await msgs.loadOlder(G, 50);
    expect(msgs.messagesFor(G)).toHaveLength(120);
  });

  it("loadUntil pages in until a quoted message is resident", async () => {
    chats.selectChat(A);
    expect(await msgs.loadUntil(A, "m0003")).toBe(true);
    expect(msgs.messagesFor(A).some((m) => m.id === "m0003")).toBe(true);
  });
});

describe("updates to non-resident messages", () => {
  beforeEach(async () => {
    await seed(Array.from({ length: 60 }, (_, i) => mk(A, i)));
    // Nothing of A is resident in memory.
  });

  it("applies receipt, edit and reaction to the stored copy", async () => {
    msgs.applyReceipt(A, ["m0000"], "read");
    msgs.applyEdit(A, "m0001", "edited", 5000);
    msgs.applyReaction(A, "m0002", "x@s.whatsapp.net", "👍");
    await vi.waitFor(async () => {
      const [m0, m1, m2] = await persist.loadLatestMessages(A, 3, 1002);
      expect(m0.status).toBe("read");
      expect(m1.text).toBe("edited");
      expect(m2.reactions).toEqual({ "x@s.whatsapp.net": "👍" });
    });
  });

  it("never downgrades a stored receipt", async () => {
    msgs.applyReceipt(A, ["m0000"], "read");
    await vi.waitFor(async () => {
      expect((await persist.loadLatestMessages(A, 1, 1000))[0].status).toBe("read");
    });
    msgs.applyReceipt(A, ["m0000"], "delivered");
    await new Promise((r) => setTimeout(r, 20));
    expect((await persist.loadLatestMessages(A, 1, 1000))[0].status).toBe("read");
  });

  it("revokes a stored message", async () => {
    msgs.applyRevoke(A, "m0000");
    await vi.waitFor(async () => {
      const [m] = await persist.loadLatestMessages(A, 1, 1000);
      expect(m.deleted).toBe(true);
      expect(m.text).toBeNull();
    });
  });

  it("inserts a tombstone when a revoked message was never stored", async () => {
    msgs.applyRevoke(A, "ghost", { senderJid: A, fromMe: false, timestamp: 2000 });
    await vi.waitFor(() => {
      expect(msgs.messagesFor(A).find((m) => m.id === "ghost")?.deleted).toBe(true);
    });
  });
});

describe("live writes", () => {
  beforeEach(() => {
    persist.enablePersistence();
  });

  it("keeps inactive chats at the resident tail but persists every message", async () => {
    for (let i = 0; i < 30; i++) msgs.addMessage(mk(C, i));
    expect(msgs.messagesFor(C)).toHaveLength(msgs.RESIDENT_TAIL);
    // Unflushed writes are already visible to reads...
    expect(await persist.loadLatestMessages(C, 100)).toHaveLength(30);
    // ...and land in IndexedDB on flush.
    await persist.flushPendingWrites();
    expect(await persist.loadLatestMessages(C, 100)).toHaveLength(30);
  });

  it("delete-for-me removes the stored row", async () => {
    await seed([mk(C, 0), mk(C, 1)]);
    msgs.removeMessage(C, "m0000");
    await persist.flushPendingWrites();
    expect((await persist.loadLatestMessages(C, 100)).map((m) => m.id)).toEqual(["m0001"]);
  });

  it("persists a confirmed send but never the optimistic bubble", async () => {
    chats.selectChat(A);
    const tmp = msgs.addOptimistic(A, "hello");
    await persist.flushPendingWrites();
    expect(await persist.loadLatestMessages(A, 10)).toHaveLength(0);

    msgs.confirmOptimistic(A, tmp, "REAL1");
    await persist.flushPendingWrites();
    expect((await persist.loadLatestMessages(A, 10)).map((m) => m.id)).toEqual(["REAL1"]);
  });

  it("queues writes made before persistence is enabled", async () => {
    await load(); // fresh modules: persistence disabled
    msgs.addMessage(mk(C, 0));
    persist.enablePersistence();
    await persist.flushPendingWrites();
    expect(await persist.loadLatestMessages(C, 10)).toHaveLength(1);
  });
});

describe("LID → PN move", () => {
  it("re-keys stored rows and unflushed writes", async () => {
    await seed(Array.from({ length: 10 }, (_, i) => mk(C, i)));
    persist.persistMessage(mk(C, 10)); // still pending
    await persist.movePersistedChat(C, E);
    await persist.flushPendingWrites();

    expect(await persist.loadLatestMessages(C, 100)).toHaveLength(0);
    const moved = await persist.loadLatestMessages(E, 100);
    expect(moved).toHaveLength(11);
    expect(moved.every((m) => m.chatJid === E)).toBe(true);
  });
});
