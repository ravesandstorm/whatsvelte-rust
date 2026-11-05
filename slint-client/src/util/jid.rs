//! JID helpers (port of `svelte-frontend/src/lib/util/jid.ts`).

pub fn jid_user(jid: &str) -> &str {
    let user = jid.split('@').next().unwrap_or(jid);
    user.split(':').next().unwrap_or(user)
}

pub fn is_group(jid: &str) -> bool {
    jid.ends_with("@g.us")
}

/// Strip the device/agent suffix so two forms of the same JID compare equal.
pub fn normalize_jid(jid: &str) -> String {
    crate::core::bridge::normalize_chat_jid(jid)
}

pub fn is_status(jid: &str) -> bool {
    jid.starts_with("status@")
}

pub fn is_status_broadcast(jid: &str) -> bool {
    jid == "status@broadcast"
}

pub fn is_newsletter(jid: &str) -> bool {
    jid.ends_with("@newsletter")
}

pub fn is_lid(jid: &str) -> bool {
    jid.ends_with("@lid")
}

/// A broadcast *list* (`<digits>@broadcast`), excluding the status feed.
pub fn is_broadcast_list(jid: &str) -> bool {
    jid.ends_with("@broadcast") && jid != "status@broadcast"
}

pub fn is_phone(jid: &str) -> bool {
    jid.ends_with("@s.whatsapp.net")
        || (!jid.contains('@') && !jid.is_empty() && jid.chars().all(|c| c.is_ascii_digit()))
}

pub fn is_self(jid: &str, self_jid: Option<&str>) -> bool {
    self_jid.is_some_and(|s| !s.is_empty() && normalize_jid(jid) == normalize_jid(s))
}

/// A regular 1:1/group conversation (not a channel, status feed or broadcast list).
pub fn is_regular_chat(jid: &str) -> bool {
    !is_newsletter(jid) && !is_status(jid) && !is_broadcast_list(jid)
}

/// Format a phone JID as `+<code> <first5> <last5>`; a LID becomes `~XXXX`.
pub fn format_phone(jid: &str) -> String {
    let user = jid_user(jid);
    if is_lid(jid) {
        let n = user.chars().count();
        return if n > 4 {
            format!("~{}", &user[user.len() - 4..])
        } else {
            format!("~{user}")
        };
    }
    let digits: String = user.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() < 10 {
        return if digits.is_empty() { user.to_string() } else { format!("+{digits}") };
    }
    let (code, national) = digits.split_at(digits.len() - 10);
    let grouped = format!("{} {}", &national[..5], &national[5..]);
    if code.is_empty() { grouped } else { format!("+{code} {grouped}") }
}

pub fn display_name(jid: &str, name: Option<&str>, push_name: Option<&str>) -> String {
    if let Some(n) = name.filter(|n| !n.is_empty()) {
        return n.to_string();
    }
    if let Some(n) = push_name.filter(|n| !n.is_empty()) {
        return n.to_string();
    }
    if is_lid(jid) || is_phone(jid) {
        return format_phone(jid);
    }
    jid_user(jid).to_string()
}

pub fn initials(label: &str) -> String {
    let parts: Vec<&str> = label.split_whitespace().collect();
    match parts.as_slice() {
        [] => "?".into(),
        [one] => one.chars().take(2).collect::<String>().to_uppercase(),
        [first, .., last] => {
            let mut s = String::new();
            s.extend(first.chars().next());
            s.extend(last.chars().next());
            s.to_uppercase()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_phones_and_lids() {
        assert_eq!(format_phone("919876543210@s.whatsapp.net"), "+91 98765 43210");
        assert_eq!(format_phone("100000012345678@lid"), "~5678");
        assert_eq!(format_phone("123@s.whatsapp.net"), "+123");
    }

    #[test]
    fn initials_work() {
        assert_eq!(initials("Ada Lovelace"), "AL");
        assert_eq!(initials("bob"), "BO");
        assert_eq!(initials("  "), "?");
    }

    #[test]
    fn normalizes_device_suffix() {
        assert_eq!(normalize_jid("12345.0:7@s.whatsapp.net"), "12345@s.whatsapp.net");
    }
}
