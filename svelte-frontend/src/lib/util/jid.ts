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

/** A privacy LID address (`<15-16 digits>@lid`) — not a real phone number. */
export function isLid(jid: string): boolean {
  return jid.endsWith("@lid");
}

/** A broadcast *list* (`<digits>@broadcast`), excluding the status feed. These
 * aren't 1:1 conversations and shouldn't clutter the main list as raw numbers. */
export function isBroadcastList(jid: string): boolean {
  return jid.endsWith("@broadcast") && jid !== "status@broadcast";
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
  // A privacy LID (`<16-digit-id>@lid`) is NOT a phone number — never format its
  // id as one (that produces a misleading fake `+code …`). Show a short neutral
  // token instead; once the LID→PN mapping is learned the caller passes a real
  // PN and this branch isn't hit.
  if (jid.endsWith("@lid")) {
    const u = jidUser(jid);
    return u.length > 4 ? `~${u.slice(-4)}` : `~${u}`;
  }
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
  // A LID has no human-meaningful number — show the short `~XXXX` token from
  // formatPhone rather than the raw 15-16 digit id (which reads as an artifact).
  if (isLid(jid)) return formatPhone(jid);
  return isPhone(jid) ? formatPhone(jid) : jidUser(jid);
}

export function initials(label: string): string {
  const parts = label.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) return "?";
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
  return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
}
