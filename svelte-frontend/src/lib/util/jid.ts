export function jidUser(jid: string): string {
  return jid.split("@")[0].split(":")[0];
}

export function isGroup(jid: string): boolean {
  return jid.endsWith("@g.us");
}

export function isStatus(jid: string): boolean {
  return jid.startsWith("status@");
}

export function displayName(
  jid: string,
  name?: string | null,
  pushName?: string | null,
): string {
  return name || pushName || jidUser(jid);
}

export function initials(label: string): string {
  const parts = label.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) return "?";
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
  return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
}
