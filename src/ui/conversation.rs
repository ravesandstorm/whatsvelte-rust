use std::collections::{HashMap, HashSet};
use dioxus::prelude::*;
use serde::Deserialize;
use crate::dto::MessageDto;
use crate::state::AppState;
use crate::ui::avatar::Avatar;
use crate::ui::jid_utils::resolve_display_name;
use crate::ui::message_list::MessageList;
use crate::ui::message_composer::MessageComposer;

#[derive(Deserialize)]
struct MessagesResponse {
    #[serde(default)]
    messages: Vec<MessageDto>,
    #[serde(rename = "hasMore", default)]
    has_more: bool,
}

#[component]
pub fn Conversation() -> Element {
    let mut app_state = use_context::<AppState>();
    let active_jid = app_state.active_chat_jid.read().clone();
    let chats = app_state.chats.read();
    let messages_map = app_state.messages.read();
    let contacts = app_state.contacts.read();

    // Dynamically load initial 50 messages from IndexedDB whenever active chat changes
    let active_jid_clone = active_jid.clone();
    use_effect(use_reactive(&active_jid_clone, move |jid_opt| {
        let mut app_state = app_state;
        if let Some(jid) = jid_opt {
            spawn(async move {
                *app_state.is_loading_messages.write() = true;
                let js = format!("return await window.whatsappDb.loadChatMessages({:?}, 50, null);", jid);
                if let Ok(eval_val) = document::eval(&js).await {
                    if let Some(json_str) = eval_val.as_str() {
                        if let Ok(resp) = serde_json::from_str::<MessagesResponse>(json_str) {
                            let mut seen = HashSet::new();
                            let mut unique_msgs = Vec::new();
                            for m in resp.messages {
                                if m.id.is_empty() || seen.insert(m.id.clone()) {
                                    unique_msgs.push(m);
                                }
                            }
                            unique_msgs.sort_by_key(|m| m.timestamp);

                            let mut map = HashMap::new();
                            map.insert(jid.clone(), unique_msgs);
                            *app_state.messages.write() = map;
                            *app_state.has_more_messages.write() = resp.has_more;
                        }
                    }
                }
                *app_state.is_loading_messages.write() = false;
            });
        } else {
            // When chat is closed, clear messages from memory to minimize RAM usage
            *app_state.messages.write() = HashMap::new();
            *app_state.has_more_messages.write() = false;
            *app_state.is_loading_messages.write() = false;
        }
    }));

    if let Some(jid) = active_jid.as_ref() {
        let active_chat = chats.iter().find(|c| c.jid == *jid);
        let contact = contacts.get(jid);
        let contact_name = contact.and_then(|c| c.verified_name.as_deref().or(c.name.as_deref()));
        let message_history = messages_map.get(jid).cloned().unwrap_or_default();
        let fallback_push = message_history.iter().find(|m| !m.from_me).and_then(|m| m.push_name.as_deref());
        let display_name = resolve_display_name(jid, contact_name.or(active_chat.and_then(|c| c.name.as_deref())), fallback_push);

        rsx! {
            div {
                style: "flex: 1 1 0; height: 100%; min-width: 0; min-height: 0; display: flex; flex-direction: column; background-color: #0b141a; overflow: hidden;",
                // Header
                div {
                    style: "height: 60px; min-height: 60px; max-height: 60px; padding: 10px 16px; background-color: #202c33; display: flex; align-items: center; justify-content: space-between; border-bottom: 1px solid #222e35; flex-shrink: 0; box-sizing: border-box;",
                    div {
                        style: "display: flex; align-items: center; gap: 12px; min-width: 0;",
                        Avatar {
                            jid: jid.clone(),
                            name: Some(display_name.clone()),
                            picture_url: contact.and_then(|c| c.picture_url.clone()),
                            size: 40,
                        }
                        div {
                            style: "display: flex; flex-direction: column; min-width: 0;",
                            span {
                                style: "font-size: 16px; font-weight: 500; color: #e9edef; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                                "{display_name}"
                            }
                        }
                    }
                    button {
                        style: "background: none; border: none; font-size: 18px; color: #8696a0; cursor: pointer; padding: 6px; border-radius: 50%; display: flex; align-items: center; justify-content: center;",
                        onclick: move |_| {
                            *app_state.active_chat_jid.write() = None;
                            *app_state.messages.write() = HashMap::new();
                        },
                        "✕"
                    }
                }
                // Message List Area (Scrollable flex item with dynamic pagination)
                MessageList { chat_jid: jid.clone(), messages: message_history }
                // Message Input Composer (Locked at bottom)
                MessageComposer {}
            }
        }
    } else {
        rsx! {
            div {
                style: "flex: 1 1 0; height: 100%; min-width: 0; min-height: 0; display: flex; flex-direction: column; align-items: center; justify-content: center; background-color: #222e35; color: #8696a0; border-bottom: 6px solid #00a884; overflow: hidden;",
                h1 { style: "font-size: 32px; font-weight: 300; color: #e9edef; margin-bottom: 12px;", "WhatsApp Desktop" }
                p { style: "font-size: 14px; color: #8696a0; max-width: 460px; text-align: center; line-height: 22px; padding: 0 16px;",
                    "Send and receive messages without keeping your phone online. Select a chat from the left sidebar to start messaging."
                }
            }
        }
    }
}
