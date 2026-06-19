// Lazy contact name/avatar cache, enriched on demand from the backend.

import { SvelteMap } from "svelte/reactivity";
import { api } from "../ipc";

export interface Contact {
  name: string | null;
  pictureUrl: string | null;
}

export const contacts = new SvelteMap<string, Contact>();
const inflight = new Set<string>();

export function contactFor(jid: string): Contact | undefined {
  return contacts.get(jid);
}

/** Fetch once per JID; cache success and failure to avoid repeat lookups. */
export async function ensureContact(jid: string) {
  if (contacts.has(jid) || inflight.has(jid)) return;
  inflight.add(jid);
  try {
    const c = await api.getContact(jid);
    contacts.set(jid, { name: c.name, pictureUrl: c.pictureUrl });
  } catch {
    contacts.set(jid, { name: null, pictureUrl: null });
  } finally {
    inflight.delete(jid);
  }
}
