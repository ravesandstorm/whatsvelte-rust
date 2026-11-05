//! Timestamp formatting for the chat list and bubbles (local time).

use chrono::{DateTime, Datelike, Local, TimeZone};

pub fn now_secs() -> i64 {
    Local::now().timestamp()
}

fn local(ts: i64) -> Option<DateTime<Local>> {
    Local.timestamp_opt(ts, 0).single()
}

/// `HH:MM` in local time.
pub fn clock(ts: i64) -> String {
    local(ts).map(|t| t.format("%H:%M").to_string()).unwrap_or_default()
}

/// Days since the epoch in local time (for grouping by day).
pub fn local_day(ts: i64) -> i64 {
    local(ts).map(|t| t.date_naive().num_days_from_ce() as i64).unwrap_or(0)
}

/// Chat-list stamp: time today, "Yesterday", else `D Mon` / `DD/MM/YY`.
pub fn list_stamp(ts: i64) -> String {
    let Some(t) = local(ts).filter(|_| ts > 0) else {
        return String::new();
    };
    let now = Local::now();
    let diff = now.date_naive().num_days_from_ce() - t.date_naive().num_days_from_ce();
    match diff {
        0 => clock(ts),
        1 => "Yesterday".into(),
        _ if t.year() == now.year() => t.format("%-d %b").to_string(),
        _ => t.format("%d/%m/%y").to_string(),
    }
}

/// Day-separator label inside a conversation.
pub fn day_label(ts: i64) -> String {
    let Some(t) = local(ts) else { return String::new() };
    let diff = Local::now().date_naive().num_days_from_ce() - t.date_naive().num_days_from_ce();
    match diff {
        0 => "Today".into(),
        1 => "Yesterday".into(),
        _ => t.format("%-d %B %Y").to_string(),
    }
}
