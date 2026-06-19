export function formatTime(epochSecs: number): string {
  if (!epochSecs) return "";
  const d = new Date(epochSecs * 1000);
  return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

/** Time for today, otherwise a short date — like the WhatsApp chat list. */
export function formatChatTime(epochSecs: number): string {
  if (!epochSecs) return "";
  const d = new Date(epochSecs * 1000);
  const now = new Date();
  if (d.toDateString() === now.toDateString()) {
    return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  }
  return d.toLocaleDateString([], { day: "2-digit", month: "2-digit", year: "2-digit" });
}
