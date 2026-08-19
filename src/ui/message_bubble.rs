use dioxus::prelude::*;
use crate::dto::MessageDto;

fn format_time(ts: i64) -> String {
    if ts <= 0 {
        return String::new();
    }
    let secs_in_day = (ts % 86400 + 86400) % 86400;
    let hours = secs_in_day / 3600;
    let mins = (secs_in_day % 3600) / 60;
    format!("{:02}:{:02}", hours, mins)
}

#[component]
pub fn MessageBubble(message: MessageDto) -> Element {
    let align_style = if message.from_me {
        "align-self: flex-end; background-color: #005c4b; color: #e9edef; border-radius: 8px 0px 8px 8px;"
    } else {
        "align-self: flex-start; background-color: #202c33; color: #e9edef; border-radius: 0px 8px 8px 8px;"
    };

    let time_str = format_time(message.timestamp);

    rsx! {
        div {
            style: "max-width: 65%; padding: 8px 12px; margin: 4px 0; font-size: 14.5px; position: relative; box-shadow: 0 1px 0.5px rgba(11,20,26,0.13); {align_style}",
            if !message.from_me {
                if let Some(sender) = message.push_name.as_ref() {
                    span {
                        style: "font-size: 12.5px; font-weight: 600; color: #53bdeb; display: block; margin-bottom: 2px;",
                        "{sender}"
                    }
                }
            }
            if let Some(text) = message.text.as_ref() {
                p { style: "margin: 0; white-space: pre-wrap; word-break: break-word;", "{text}" }
            }
            div {
                style: "display: flex; justify-content: flex-end; align-items: center; gap: 4px; margin-top: 4px;",
                if !time_str.is_empty() {
                    span { style: "font-size: 11px; color: #8696a0;", "{time_str}" }
                }
                if message.from_me {
                    span { style: "font-size: 11px; color: #53bdeb;", "✓✓" }
                }
            }
        }
    }
}
