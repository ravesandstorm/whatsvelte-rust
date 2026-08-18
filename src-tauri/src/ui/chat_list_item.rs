use dioxus::prelude::*;
use crate::dto::ChatDto;
use crate::state::AppState;

#[component]
pub fn ChatListItem(chat: ChatDto) -> Element {
    let mut app_state = use_context::<AppState>();
    let active_jid = app_state.active_chat_jid.read();
    let is_active = active_jid.as_deref() == Some(&chat.jid);
    
    let background = if is_active { "#2a3942" } else { "transparent" };
    let chat_clone = chat.clone();
    let jid_clone = chat.jid.clone();

    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 15px; padding: 12px 16px; cursor: pointer; background-color: {background}; border-bottom: 1px solid #222e35; transition: background-color 0.15s ease;",
            onclick: move |_| {
                *app_state.active_chat_jid.write() = Some(jid_clone.clone());
            },
            // Avatar
            div {
                style: "width: 48px; height: 48px; border-radius: 50%; background-color: #6b7c85; display: flex; align-items: center; justify-content: center; font-weight: 600; color: #ffffff; font-size: 18px;",
                "{chat_clone.name.as_deref().and_then(|n| n.chars().next()).unwrap_or('?')}"
            }
            // Chat Details
            div {
                style: "flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 4px;",
                div {
                    style: "display: flex; justify-content: space-between; align-items: center;",
                    span {
                        style: "font-size: 16px; font-weight: 500; color: #e9edef; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                        "{chat_clone.name.as_deref().unwrap_or(&chat_clone.jid)}"
                    }
                    if chat_clone.timestamp > 0 {
                        span {
                            style: "font-size: 12px; color: #8696a0;",
                            "12:00"
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
                            style: "background-color: #00a884; color: #111b21; font-size: 12px; font-weight: bold; border-radius: 10px; padding: 2px 6px; min-width: 18px; text-align: center;",
                            "{chat_clone.unread}"
                        }
                    }
                }
            }
        }
    }
}
