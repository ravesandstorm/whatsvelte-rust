//! Utilities for resolving WhatsApp JIDs to human-friendly display names and phone numbers.

/// Clean and format a phone number with country code and readable grouping.
pub fn format_phone_number(phone: &str) -> String {
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return phone.to_string();
    }

    // US / Canada: 1 + 10 digits -> +1 (XXX) XXX-XXXX
    if digits.len() == 11 && digits.starts_with('1') {
        return format!("+1 ({}) {}-{}", &digits[1..4], &digits[4..7], &digits[7..]);
    }
    // US / Canada 10 digits (assumed local or +1)
    if digits.len() == 10 && !digits.starts_with('0') {
        return format!("+1 ({}) {}-{}", &digits[0..3], &digits[3..6], &digits[6..]);
    }
    // India: 91 + 10 digits -> +91 XXXXX XXXXX
    if digits.len() == 12 && digits.starts_with("91") {
        return format!("+91 {} {}", &digits[2..7], &digits[7..]);
    }
    // UK: 44 + 10 digits -> +44 XXXX XXXXXX
    if digits.len() >= 11 && digits.starts_with("44") {
        return format!("+44 {} {}", &digits[2..6], &digits[6..]);
    }
    // Brazil: 55 + 10/11 digits
    if (digits.len() == 12 || digits.len() == 13) && digits.starts_with("55") {
        return format!("+55 {} {}", &digits[2..4], &digits[4..]);
    }

    // Generic international format
    if digits.len() > 8 {
        let (cc, rest) = if digits.len() > 11 {
            digits.split_at(2)
        } else if digits.len() > 9 {
            digits.split_at(1)
        } else {
            ("", digits.as_str())
        };
        if !cc.is_empty() {
            let mid = rest.len() / 2;
            format!("+{} {} {}", cc, &rest[..mid], &rest[mid..])
        } else {
            let mid = digits.len() / 2;
            format!("+{} {}", &digits[..mid], &digits[mid..])
        }
    } else {
        format!("+{}", digits)
    }
}

/// Resolves a JID, explicit chat name, and optional fallback push name into a clean user-facing title.
pub fn resolve_display_name(
    jid: &str,
    explicit_name: Option<&str>,
    push_name_fallback: Option<&str>,
) -> String {
    // 1. If explicit name is present, valid, and not just a raw JID
    if let Some(name) = explicit_name {
        let trimmed = name.trim();
        if !trimmed.is_empty()
            && !trimmed.ends_with("@s.whatsapp.net")
            && !trimmed.ends_with("@lid")
            && !trimmed.ends_with("@g.us")
            && !trimmed.ends_with("@broadcast")
        {
            return trimmed.to_string();
        }
    }

    // 2. If fallback push name is present, valid, and not a raw JID
    if let Some(push) = push_name_fallback {
        let trimmed = push.trim();
        if !trimmed.is_empty()
            && !trimmed.ends_with("@s.whatsapp.net")
            && !trimmed.ends_with("@lid")
            && !trimmed.ends_with("@g.us")
        {
            return trimmed.to_string();
        }
    }

    // 3. Status updates broadcast
    if jid == "status@broadcast" {
        return "Status".to_string();
    }

    // 4. Standard phone number JID: <countrycode><number>@s.whatsapp.net
    if let Some(phone) = jid.strip_suffix("@s.whatsapp.net") {
        return format_phone_number(phone);
    }

    // 5. Group JID: <group_id>@g.us
    if let Some(group_id) = jid.strip_suffix("@g.us") {
        if let Some(name) = explicit_name {
            let trimmed = name.trim();
            if !trimmed.is_empty() && !trimmed.ends_with("@g.us") {
                return trimmed.to_string();
            }
        }
        let short = if group_id.len() > 8 { &group_id[..8] } else { group_id };
        return format!("Group ({})", short);
    }

    // 6. LID JID (Linked Identity): <lid_digits>@lid
    if let Some(lid) = jid.strip_suffix("@lid") {
        let short = if lid.len() > 6 { &lid[..6] } else { lid };
        return format!("User ({})", short);
    }

    // 7. General user@server JID
    if let Some((user, _)) = jid.split_once('@') {
        if user.chars().all(|c| c.is_ascii_digit()) {
            return format_phone_number(user);
        }
        return user.to_string();
    }

    // Fallback
    if jid.chars().all(|c| c.is_ascii_digit()) {
        format_phone_number(jid)
    } else {
        jid.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phone_number_formatting() {
        assert_eq!(format_phone_number("12025550123"), "+1 (202) 555-0123");
        assert_eq!(format_phone_number("919876543210"), "+91 98765 43210");
        assert_eq!(format_phone_number("447911123456"), "+44 7911 123456");
    }

    #[test]
    fn test_resolve_display_name() {
        assert_eq!(
            resolve_display_name("12025550123@s.whatsapp.net", Some("Alice Smith"), None),
            "Alice Smith"
        );
        assert_eq!(
            resolve_display_name("12025550123@s.whatsapp.net", None, Some("Bob")),
            "Bob"
        );
        assert_eq!(
            resolve_display_name("12025550123@s.whatsapp.net", None, None),
            "+1 (202) 555-0123"
        );
        assert_eq!(
            resolve_display_name("123456789012345@lid", None, Some("Charlie")),
            "Charlie"
        );
        assert_eq!(
            resolve_display_name("123456789012345@lid", None, None),
            "User (123456)"
        );
        assert_eq!(
            resolve_display_name("120363023456789@g.us", Some("Family Group"), None),
            "Family Group"
        );
    }
}
