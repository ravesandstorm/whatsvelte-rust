export function jidUser(jid: string): string {
  return jid.split("@")[0].split(":")[0];
}

export function isGroup(jid: string): boolean {
  return jid.endsWith("@g.us");
}

/** Strip the device/agent suffix so two forms of the same JID compare equal. */
export function normalizeJid(jid: string): string {
  const at = jid.indexOf("@");
  if (at < 0) return jid;
  const user = jid.slice(0, at).split(":")[0].split(".")[0];
  return user + jid.slice(at);
}

export function isStatus(jid: string): boolean {
  return jid.startsWith("status@");
}

/** The single status-broadcast feed JID. */
export function isStatusBroadcast(jid: string): boolean {
  return jid === "status@broadcast";
}

/** Channels / newsletters (non-phone broadcast chats). */
export function isNewsletter(jid: string): boolean {
  return jid.endsWith("@newsletter");
}

/** A real phone-number JID (vs group/newsletter/status/lid). */
export function isPhone(jid: string): boolean {
  return jid.endsWith("@s.whatsapp.net") || (!jid.includes("@") && /^\d+$/.test(jid));
}

/** True when `jid` is the logged-in account. */
export function isSelf(jid: string, selfJid: string | null | undefined): boolean {
  return !!selfJid && normalizeJid(jid) === normalizeJid(selfJid);
}

/** Format a phone JID as `+<code> <first5> <last5>` (e.g. +91 98765 43210).
 * The national part is the last 10 digits; anything before it is the country
 * code. Falls back to the raw user part when there aren't enough digits. */
export function formatPhone(jid: string): string {
  const digits = jidUser(jid).replace(/\D/g, "");
  if (digits.length < 10) return digits ? `+${digits}` : jidUser(jid);
  const national = digits.slice(-10);
  const code = digits.slice(0, -10);
  const grouped = `${national.slice(0, 5)} ${national.slice(5)}`;
  return code ? `+${code} ${grouped}` : grouped;
}

export function displayName(
  jid: string,
  name?: string | null,
  pushName?: string | null,
): string {
  if (name) return name;
  if (pushName) return pushName;
  return isPhone(jid) ? formatPhone(jid) : jidUser(jid);
}

export function initials(label: string): string {
  const parts = label.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) return "?";
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
  return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
}
