// Transient composer state shared between a message bubble (which starts a
// reply) and the composer (which renders the reply banner and sends it).

import type { UiMessage } from "./messages.svelte";

export const compose = $state({
  /** The message currently being replied to, or null. */
  replyTarget: null as UiMessage | null,
  /** The message currently being edited, or null. */
  editTarget: null as UiMessage | null,
});

export function startReply(target: UiMessage) {
  compose.editTarget = null;
  compose.replyTarget = target;
}

export function cancelReply() {
  compose.replyTarget = null;
}

export function startEdit(target: UiMessage) {
  compose.replyTarget = null;
  compose.editTarget = target;
}

export function cancelEdit() {
  compose.editTarget = null;
}
