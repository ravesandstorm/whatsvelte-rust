use dioxus::prelude::*;
use crate::dto::ChatDto;
use crate::state::AppState;
use crate::ui::avatar::Avatar;
use crate::ui::jid_utils::resolve_display_name;

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
pub fn ChatListItem(chat: ChatDto) -> Element {
    let mut app_state = use_context::<AppState>();
    let active_jid = app_state.active_chat_jid.read();
    let is_active = active_jid.as_deref() == Some(&chat.jid);
    let contacts = app_state.contacts.read();
    
    let background = if is_active { "#2a3942" } else { "transparent" };
    let chat_clone = chat.clone();
    let jid_clone = chat.jid.clone();
    let time_str = format_time(chat.timestamp);

    let contact = contacts.get(&chat.jid);
    let contact_name = contact.and_then(|c| c.verified_name.as_deref().or(c.name.as_deref()));
    let display_name = resolve_display_name(&chat.jid, contact_name.or(chat.name.as_deref()), None);

    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 15px; padding: 12px 16px; cursor: pointer; background-color: {background}; border-bottom: 1px solid #222e35; transition: background-color 0.15s ease;",
            onclick: move |_| {
                *app_state.active_chat_jid.write() = Some(jid_clone.clone());
            },
            // Avatar
            Avatar {
                jid: chat_clone.jid.clone(),
                name: Some(display_name.clone()),
                picture_url: contact.and_then(|c| c.picture_url.clone()),
                size: 48,
            }
            // Chat Details
            div {
                style: "flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 4px;",
                div {
                    style: "display: flex; justify-content: space-between; align-items: center;",
                    span {
                        style: "font-size: 16px; font-weight: 500; color: #e9edef; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                        "{display_name}"
                    }
                    if !time_str.is_empty() {
                        span {
                            style: "font-size: 12px; color: #8696a0; flex-shrink: 0; margin-left: 8px;",
                            "{time_str}"
                        }
                    }
                }
                div {
                    style: "display: flex; justify-content: space-between; align-items: center;",
                    p {
                        style: "font-size: 14px; color: #8696a0; margin: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                        "{chat_clone.last_message.as_deref().unwrap_or(\"\")}"
                    }
                    if chat_clone.unread > 0 {
                        span {
                            style: "background-color: #00a884; color: #111b21; font-size: 12px; font-weight: bold; border-radius: 10px; padding: 2px 6px; min-width: 18px; text-align: center; flex-shrink: 0; margin-left: 6px;",
                            "{chat_clone.unread}"
                        }
                    }
                }
            }
        }
    }
}
