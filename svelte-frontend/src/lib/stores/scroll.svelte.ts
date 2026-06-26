// Cross-component request to scroll the active message list to a specific
// message (used when tapping a reply's quoted preview). MessageList owns the
// scroll container and watches `nonce` to react to each request.

export const scrollTarget = $state({ id: null as string | null, nonce: 0 });

export function scrollToMessage(id: string) {
  scrollTarget.id = id;
  scrollTarget.nonce++;
}

// Disarm the request once the jump (and its highlight) is done, so a stale id
// can't pull the viewport back to an old message on later renders.
export function clearScrollTarget() {
  scrollTarget.id = null;
}
