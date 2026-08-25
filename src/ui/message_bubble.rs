use dioxus::prelude::*;
use crate::dto::MessageDto;
use crate::ui::jid_utils::resolve_display_name;
use crate::ui::time_utils::format_message_time;

#[component]
pub fn MessageBubble(message: MessageDto) -> Element {
    let align_style = if message.from_me {
        "align-self: flex-end; background-color: #005c4b; color: #e9edef; border-radius: 8px 0px 8px 8px;"
    } else {
        "align-self: flex-start; background-color: #202c33; color: #e9edef; border-radius: 0px 8px 8px 8px;"
    };

    let time_str = format_message_time(message.timestamp);

    let sender_display = if !message.from_me {
        let is_group = message.chat_jid.ends_with("@g.us");
        if is_group || message.push_name.is_some() {
            Some(resolve_display_name(&message.sender_jid, None, message.push_name.as_deref()))
        } else {
            None
        }
    } else {
        None
    };

    let status_str = message.status.as_deref().unwrap_or("sent");

    rsx! {
        div {
            style: "max-width: 65%; padding: 8px 12px; margin: 4px 0; font-size: 14.5px; position: relative; box-shadow: 0 1px 0.5px rgba(11,20,26,0.13); {align_style}",
            if let Some(sender) = sender_display {
                span {
                    style: "font-size: 12.5px; font-weight: 600; color: #53bdeb; display: block; margin-bottom: 2px;",
                    "{sender}"
                }
            }
            if let Some(thumb) = message.thumbnail.as_ref().filter(|t| !t.is_empty()) {
                img {
                    src: "data:image/jpeg;base64,{thumb}",
                    alt: "Photo",
                    style: "max-width: 100%; max-height: 280px; border-radius: 6px; object-fit: contain; margin-bottom: 6px; display: block; background-color: #111b21;",
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
                    match status_str {
                        "read" | "played" => rsx! { span { style: "font-size: 11px; color: #53bdeb;", "✓✓" } },
                        "delivered" => rsx! { span { style: "font-size: 11px; color: #8696a0;", "✓✓" } },
                        "pending" => rsx! { span { style: "font-size: 10px; color: #8696a0;", "🕒" } },
                        _ => rsx! { span { style: "font-size: 11px; color: #8696a0;", "✓" } },
                    }
                }
            }
        }
    }
}
