// Short-lived error notices for actions that failed after an optimistic UI
// update (a send, edit, reaction or delete the server rejected).

export interface Toast {
  id: number;
  text: string;
}

export const toasts = $state<Toast[]>([]);

let seq = 0;
const TTL_MS = 5000;
const MAX = 3;

export function showToast(text: string) {
  // Collapse repeats (e.g. several sends failing while offline).
  if (toasts.some((t) => t.text === text)) return;
  const id = ++seq;
  toasts.push({ id, text });
  while (toasts.length > MAX) toasts.shift();
  setTimeout(() => dismissToast(id), TTL_MS);
}

export function dismissToast(id: number) {
  const i = toasts.findIndex((t) => t.id === id);
  if (i >= 0) toasts.splice(i, 1);
}
