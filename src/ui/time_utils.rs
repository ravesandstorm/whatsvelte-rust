use chrono::{DateTime, Local};

/// Normalizes a timestamp that could be in seconds or milliseconds into unix seconds.
pub fn normalize_timestamp(raw_ts: i64) -> i64 {
    if raw_ts > 100_000_000_000 {
        raw_ts / 1000
    } else {
        raw_ts
    }
}

/// Formats a chat preview time:
/// - If message is > 1 day old (> 24 hours or previous calendar day), returns ONLY Date: `DD/MM/YYYY`
/// - Otherwise returns ONLY Time: `HH:MM` (24-hour format)
pub fn format_chat_time(raw_ts: i64) -> String {
    if raw_ts <= 0 {
        return String::new();
    }
    let ts = normalize_timestamp(raw_ts);
    let msg_time = match DateTime::from_timestamp(ts, 0) {
        Some(dt) => dt.with_timezone(&Local),
        None => return String::new(),
    };

    let now = Local::now();
    let duration = now.signed_duration_since(msg_time);

    if duration.num_days() >= 1 || msg_time.date_naive() != now.date_naive() {
        msg_time.format("%d/%m/%Y").to_string()
    } else {
        msg_time.format("%H:%M").to_string()
    }
}

/// Formats a message bubble time (always `HH:MM`).
pub fn format_message_time(raw_ts: i64) -> String {
    if raw_ts <= 0 {
        return String::new();
    }
    let ts = normalize_timestamp(raw_ts);
    match DateTime::from_timestamp(ts, 0) {
        Some(dt) => dt.with_timezone(&Local).format("%H:%M").to_string(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_timestamp() {
        assert_eq!(normalize_timestamp(1724581234), 1724581234);
        assert_eq!(normalize_timestamp(1724581234000), 1724581234);
    }

    #[test]
    fn test_format_chat_time_today() {
        let now_ts = Local::now().timestamp();
        let formatted = format_chat_time(now_ts);
        assert_eq!(formatted.len(), 5); // HH:MM
        assert!(formatted.contains(':'));
    }

    #[test]
    fn test_format_chat_time_past_date() {
        // Jan 15 2024
        let past_ts = 1705320000;
        let formatted = format_chat_time(past_ts);
        assert!(formatted.contains('/'));
    }
}
