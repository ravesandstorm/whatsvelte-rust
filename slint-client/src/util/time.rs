//! Timestamp formatting for the chat list and bubbles (local time).

use chrono::{DateTime, Local, TimeZone};

pub fn now_secs() -> i64 {
    Local::now().timestamp()
}

fn local(ts: i64) -> Option<DateTime<Local>> {
    Local.timestamp_opt(ts, 0).single()
}

/// `HH:MM` in local time.
pub fn clock(ts: i64) -> String {
    local(ts)
        .map(|t| t.format("%H:%M").to_string())
        .unwrap_or_default()
}

/// Chat-list stamp (formatChatTime): time today, else `DD/MM/YY`.
pub fn list_stamp(ts: i64) -> String {
    let Some(t) = local(ts).filter(|_| ts > 0) else {
        return String::new();
    };
    if t.date_naive() == Local::now().date_naive() {
        clock(ts)
    } else {
        t.format("%d/%m/%y").to_string()
    }
}
