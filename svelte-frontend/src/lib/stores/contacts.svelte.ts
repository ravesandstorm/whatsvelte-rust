// Lazy contact name/avatar cache, enriched on demand from the backend.
//
// The backend exposes no saved-contact-name API — regular display names only
// reach us as `pushName` on messages and `conv.name` in history sync. Those are
// transient, so we additionally keep a *persisted* name cache (localStorage,
// account-keyed, same pattern as stores/lid.ts) that survives restarts. Learned
// names are written straight into the reactive `contacts` map so the UI updates,
// and mirrored into the persisted cache for the next launch.

import { SvelteMap } from "svelte/reactivity";
import { api } from "../ipc";
import { normalizeJid } from "../util/jid";

export interface Contact {
  name: string | null;
  verifiedName: string | null;
  lid: string | null;
  pictureUrl: string | null;
}

export const contacts = new SvelteMap<string, Contact>();
const inflight = new Set<string>();

/** Persisted learned names: normalizeJid → name. Plain map; the reactive copy
 * lives in `contacts`. */
const nameCache = new Map<string, string>();
let lsKey: string | null = null;
let saveTimer: ReturnType<typeof setTimeout> | null = null;

export function contactFor(jid: string): Contact | undefined {
  return contacts.get(jid) ?? contacts.get(normalizeJid(jid));
}

/** Load the persisted name cache for an account and seed the reactive map.
 * Must run on boot (before/with hydration) so names render without a re-fetch. */
export function loadNameCache(account: string | null) {
  nameCache.clear();
  lsKey = account ? `name-cache:${account}` : null;
  if (!lsKey) return;
  try {
    const raw = localStorage.getItem(lsKey);
    if (!raw) return;
    for (const [jid, name] of Object.entries(JSON.parse(raw) as Record<string, string>)) {
      nameCache.set(jid, name);
      if (!contacts.get(jid)?.name) {
        contacts.set(jid, {
          name,
          verifiedName: contacts.get(jid)?.verifiedName ?? null,
          lid: contacts.get(jid)?.lid ?? null,
          pictureUrl: contacts.get(jid)?.pictureUrl ?? null,
        });
      }
    }
  } catch (e) {
    console.error("load name cache failed", e);
  }
}

/** Forget learned names (logout / different account linked). */
export function clearNameCache() {
  if (lsKey) {
    try {
      localStorage.removeItem(lsKey);
    } catch {
      /* ignore */
    }
  }
  nameCache.clear();
  lsKey = null;
}

function scheduleSave() {
  if (!lsKey || saveTimer) return;
  saveTimer = setTimeout(() => {
    saveTimer = null;
    if (!lsKey) return;
    try {
      localStorage.setItem(lsKey, JSON.stringify(Object.fromEntries(nameCache)));
    } catch (e) {
      console.error("save name cache failed", e);
    }
  }, 800);
}

/** Record a display name learned from a pushName / history conv.name / contact
 * lookup. Keyed by normalized JID; fills in only when we don't already have a
 * name, so an explicit contact name is never clobbered by a later pushName. */
export function learnName(jid: string | null | undefined, name: string | null | undefined) {
  if (!jid || !name) return;
  const key = normalizeJid(jid);
  if (!key || key === name) return;
  if (nameCache.get(key) === name && contacts.get(key)?.name) return;
  nameCache.set(key, name);
  scheduleSave();
  const existing = contacts.get(key);
  if (!existing?.name) {
    contacts.set(key, {
      name,
      verifiedName: existing?.verifiedName ?? null,
      lid: existing?.lid ?? null,
      pictureUrl: existing?.pictureUrl ?? null,
    });
  }
}

/** Fetch once per JID; cache success and failure to avoid repeat lookups.
 * Preserves any already-learned name when the backend has none. */
export async function ensureContact(jid: string) {
  if (contacts.has(jid) || inflight.has(jid)) return;
  inflight.add(jid);
  const learned = contactFor(jid)?.name ?? null;
  try {
    const c = await api.getContact(jid);
    const name = c.name ?? learned;
    contacts.set(jid, {
      name,
      verifiedName: c.verifiedName,
      lid: c.lid,
      pictureUrl: c.pictureUrl,
    });
    if (c.name) learnName(jid, c.name);
  } catch {
    contacts.set(jid, { name: learned, verifiedName: null, lid: null, pictureUrl: null });
  } finally {
    inflight.delete(jid);
  }
}
