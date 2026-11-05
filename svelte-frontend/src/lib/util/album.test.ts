import { describe, expect, it } from "vitest";
import { groupAlbums } from "./album";
import type { UiMessage } from "../stores/messages.svelte";

const media = {
  kind: "image",
  descriptor: {
    directPath: "",
    mediaKey: "",
    fileSha256: "",
    fileEncSha256: "",
    fileLength: 0,
    mediaType: "image",
  },
  mimetype: "image/jpeg",
  fileName: null,
  durationSecs: null,
} as unknown as UiMessage["media"];

let seq = 0;
function msg(extra: Partial<UiMessage> = {}): UiMessage {
  seq++;
  return {
    id: "m" + seq,
    chatJid: "a@s.whatsapp.net",
    senderJid: "a@s.whatsapp.net",
    fromMe: false,
    timestamp: 1000 + seq,
    pushName: null,
    text: null,
    kind: "image",
    thumbnail: null,
    media,
    quoted: null,
    ...extra,
  };
}

const shape = (rows: ReturnType<typeof groupAlbums>) =>
  rows.map((r) => (r.type === "album" ? `A${r.items.length}` : "m"));

describe("groupAlbums", () => {
  it("groups items sharing an album parent, even just two", () => {
    const rows = groupAlbums([
      msg({ kind: "text", media: null, text: "hi" }),
      msg({ albumParentId: "P" }),
      msg({ albumParentId: "P", kind: "video" }),
      msg({ albumParentId: "Q" }),
    ]);
    expect(shape(rows)).toEqual(["m", "A2", "m"]);
  });

  it("infers an album from 4+ uncaptioned photos sent together", () => {
    expect(shape(groupAlbums([msg(), msg(), msg()]))).toEqual(["m", "m", "m"]);
    expect(shape(groupAlbums([msg(), msg(), msg(), msg(), msg()]))).toEqual(["A5"]);
  });

  it("does not infer across senders, captions, long gaps or replies", () => {
    const other = { senderJid: "b@s.whatsapp.net" };
    expect(shape(groupAlbums([msg(), msg(), msg(other), msg(), msg()]))).toEqual([
      "m",
      "m",
      "m",
      "m",
      "m",
    ]);
    expect(shape(groupAlbums([msg(), msg(), msg({ text: "cap" }), msg(), msg()]))).toEqual([
      "m",
      "m",
      "m",
      "m",
      "m",
    ]);
    const late = msg({ timestamp: 99999 });
    expect(shape(groupAlbums([msg(), msg(), msg(), late, msg({ timestamp: 99999 })]))).toEqual([
      "m",
      "m",
      "m",
      "m",
      "m",
    ]);
  });

  it("never groups deleted or still-sending items", () => {
    const rows = groupAlbums([msg(), msg(), msg({ deleted: true }), msg(), msg()]);
    expect(rows.every((r) => r.type === "msg")).toBe(true);
    const pend = groupAlbums([msg(), msg(), msg(), msg({ id: "tmp-1" })]);
    expect(pend.every((r) => r.type === "msg")).toBe(true);
  });
});
