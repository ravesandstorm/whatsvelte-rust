// Group consecutive photos/videos into albums for display, WhatsApp-style.
//
// Album items sent by current clients carry `albumParentId` (the album header
// message's id, from messageContextInfo.messageAssociation). Older clients and
// some history-sync rows don't, so a run of uncaptioned photos/videos from the
// same sender sent close together is inferred to be an album too.

import type { UiMessage } from "../stores/messages.svelte";

export type Row =
  | { type: "msg"; key: string; m: UiMessage }
  | { type: "album"; key: string; items: UiMessage[] };

/** Inferred albums need at least this many items (WhatsApp's threshold). */
const MIN_INFERRED = 4;
/** Max gap between consecutive items of an inferred album, in seconds. */
const MAX_GAP_SECS = 60;

function albumable(m: UiMessage): boolean {
  return (
    (m.kind === "image" || m.kind === "video") &&
    !!m.media &&
    !m.deleted &&
    !m.quoted &&
    !m.id.startsWith("tmp-")
  );
}

function sameSender(a: UiMessage, b: UiMessage): boolean {
  return a.fromMe === b.fromMe && a.senderJid === b.senderJid;
}

export function groupAlbums(list: UiMessage[]): Row[] {
  const rows: Row[] = [];
  let i = 0;
  while (i < list.length) {
    const m = list[i];
    let j = i + 1;
    if (albumable(m) && m.albumParentId) {
      // Explicit album: every following item with the same parent.
      while (
        j < list.length &&
        albumable(list[j]) &&
        list[j].albumParentId === m.albumParentId &&
        sameSender(m, list[j])
      )
        j++;
      if (j - i >= 2) {
        rows.push({ type: "album", key: "album:" + m.id, items: list.slice(i, j) });
        i = j;
        continue;
      }
    } else if (albumable(m) && !m.text) {
      // Inferred album: a burst of uncaptioned items from one sender.
      while (
        j < list.length &&
        albumable(list[j]) &&
        !list[j].albumParentId &&
        !list[j].text &&
        sameSender(m, list[j]) &&
        list[j].timestamp - list[j - 1].timestamp <= MAX_GAP_SECS
      )
        j++;
      if (j - i >= MIN_INFERRED) {
        rows.push({ type: "album", key: "album:" + m.id, items: list.slice(i, j) });
        i = j;
        continue;
      }
    }
    rows.push({ type: "msg", key: m.id, m });
    i++;
  }
  return rows;
}
