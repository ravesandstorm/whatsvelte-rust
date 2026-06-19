// Transient composer state shared between a message bubble (which starts a
// reply) and the composer (which renders the reply banner and sends it).

import type { UiMessage } from "./messages.svelte";

export const compose = $state({
  /** The message currently being replied to, or null. */
  replyTarget: null as UiMessage | null,
});

export function startReply(target: UiMessage) {
  compose.replyTarget = target;
}

export function cancelReply() {
  compose.replyTarget = null;
}
