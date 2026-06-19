// LID ↔ phone-number unification.
//
// WhatsApp addresses the same contact two ways: a phone-number JID
// (`<user>@s.whatsapp.net`, "PN") and a privacy LID (`<user>@lid`). The server
// can switch a live conversation from one form to the other mid-stream, which
// otherwise splits a single chat into two — one with history/profile (PN) and a
// twin where new messages + receipts land (LID).
//
// We unify on the PN form (it carries the history-sync name/profile). The
// learned `lid → pn` map lets every ingestion point canonicalize a JID
// *synchronously*, so once a mapping is known no future LID event can re-split
// the conversation. Resolution is retryable: a miss (mapping not learned yet) is
// never cached, so a later event re-attempts it.

import { api } from "../ipc";
import { chats, mergeChats } from "./chats.svelte";
import { messagesByChat, mergeMessages } from "./messages.svelte";
import { deletePersisted, schedulePersistChats, schedulePersistMessages } from "../persist";

/** Learned canonical mapping: `<user>@lid` → `<user>@s.whatsapp.net`. */
const lidToPn = new Map<string, string>();
/** LIDs with a resolution request in flight, to avoid duplicate IPC calls. */
const inFlight = new Set<string>();

let lsKey: string | null = null;

/** Map a JID to its canonical (PN) form if a mapping is known; otherwise return
 * it unchanged. Synchronous — safe to call on every ingested event. */
export function canonicalJid(jid: string): string {
  if (!jid.endsWith("@lid")) return jid;
  return lidToPn.get(jid) ?? jid;
}

/** Load the persisted map for an account (keyed so a re-link can't inherit a
 * stale map). Must run before hydration so canonicalJid works on boot. */
export function loadLidMap(account: string | null) {
  lidToPn.clear();
  lsKey = account ? `lid-pn-map:${account}` : null;
  if (!lsKey) return;
  try {
    const raw = localStorage.getItem(lsKey);
    if (!raw) return;
    for (const [lid, pn] of Object.entries(JSON.parse(raw) as Record<string, string>)) {
      lidToPn.set(lid, pn);
    }
  } catch (e) {
    console.error("load lid map failed", e);
  }
}

/** Forget all mappings (logout / different account linked). */
export function clearLidMap() {
  if (lsKey) {
    try {
      localStorage.removeItem(lsKey);
    } catch {
      /* ignore */
    }
  }
  lidToPn.clear();
  lsKey = null;
}

function saveLidMap() {
  if (!lsKey) return;
  try {
    localStorage.setItem(lsKey, JSON.stringify(Object.fromEntries(lidToPn)));
  } catch (e) {
    console.error("save lid map failed", e);
  }
}

/** Merge a split LID conversation into its PN form (chat row + messages, memory
 * + IndexedDB). No-op when nothing is split under the LID key. */
function applyMerge(lid: string, pn: string) {
  if (!chats.has(lid) && !messagesByChat.has(lid)) return;
  mergeMessages(lid, pn);
  mergeChats(lid, pn);
  void deletePersisted(lid);
  schedulePersistChats();
  schedulePersistMessages(pn);
}

/** Ensure `jid` (if a LID) is unified with its PN form. Learns the mapping on
 * demand and merges any already-split conversation. Idempotent and retryable —
 * call it for every LID JID seen on an incoming event. */
export async function unifyLid(jid: string): Promise<void> {
  if (!jid.endsWith("@lid")) return;

  const known = lidToPn.get(jid);
  if (known) {
    applyMerge(jid, known); // re-merge if a twin re-appeared
    return;
  }

  if (inFlight.has(jid)) return;
  inFlight.add(jid);
  try {
    const r = await api.resolveJid(jid);
    // Miss: mapping not learned yet. Do NOT cache — a later event retries.
    if (!r.pn || r.pn === jid) return;
    lidToPn.set(jid, r.pn);
    saveLidMap();
    applyMerge(jid, r.pn);
  } catch (e) {
    console.error("lid unify failed", e);
  } finally {
    inFlight.delete(jid);
  }
}
