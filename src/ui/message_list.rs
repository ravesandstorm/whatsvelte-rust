use dioxus::prelude::*;
use serde::Deserialize;
use crate::dto::MessageDto;
use crate::state::AppState;
use crate::ui::message_bubble::MessageBubble;

#[derive(Deserialize)]
struct MessagesResponse {
    #[serde(default)]
    messages: Vec<MessageDto>,
    #[serde(rename = "hasMore", default)]
    has_more: bool,
}

#[component]
pub fn MessageList(chat_jid: String, messages: Vec<MessageDto>) -> Element {
    let app_state = use_context::<AppState>();
    let msg_count = messages.len();
    let is_loading = *app_state.is_loading_messages.read();
    let has_more = *app_state.has_more_messages.read();

    // Auto-scroll to bottom only when initial messages load or newest message is appended
    use_effect(use_reactive(&msg_count, move |_| {
        let _ = document::eval(
            r#"
            (function() {
                var el = document.getElementById('message-list-scroll');
                if (el && (!el.dataset.scrolledUp || el.dataset.scrolledUp === 'false')) {
                    el.scrollTo({ top: el.scrollHeight, behavior: 'smooth' });
                }
            })();
            "#
        );
    }));

    let load_older = {
        let chat_jid = chat_jid.clone();
        let oldest_ts = messages.first().map(|m| m.timestamp).unwrap_or(0);
        move |_| {
            if is_loading || !has_more || oldest_ts <= 0 {
                return;
            }
            let jid = chat_jid.clone();
            let mut app_state = app_state;
            spawn(async move {
                *app_state.is_loading_messages.write() = true;
                let js = format!("return await window.whatsappDb.loadChatMessages({:?}, 50, {});", jid, oldest_ts);
                if let Ok(eval_val) = document::eval(&js).await {
                    if let Some(json_str) = eval_val.as_str() {
                        if let Ok(resp) = serde_json::from_str::<MessagesResponse>(json_str) {
                            if !resp.messages.is_empty() {
                                let mut map = app_state.messages.write();
                                if let Some(current) = map.get_mut(&jid) {
                                    let mut new_list = resp.messages;
                                    new_list.append(current);
                                    new_list.sort_by_key(|m| m.timestamp);
                                    new_list.dedup_by(|a, b| a.id == b.id);
                                    *current = new_list;
                                }
                            }
                            *app_state.has_more_messages.write() = resp.has_more;
                        }
                    }
                }
                *app_state.is_loading_messages.write() = false;
            });
        }
    };

    rsx! {
        div {
            id: "message-list-scroll",
            style: "flex: 1 1 0; min-height: 0; height: 0; overflow-y: auto; overflow-x: hidden; scroll-behavior: smooth; padding: 20px 32px; display: flex; flex-direction: column; gap: 6px; background-color: #0b141a; background-image: radial-gradient(#1f2c34 1px, transparent 0); background-size: 20px 20px; box-sizing: border-box;",
            onscroll: move |_| {
                let _ = document::eval(
                    r#"
                    var el = document.getElementById('message-list-scroll');
                    if (el) {
                        el.dataset.scrolledUp = (el.scrollHeight - el.scrollTop - el.clientHeight > 150) ? 'true' : 'false';
                    }
                    "#
                );
            },
            if has_more {
                div {
                    style: "display: flex; justify-content: center; padding: 8px 0;",
                    button {
                        style: "background-color: #202c33; border: 1px solid #374248; border-radius: 16px; color: #53bdeb; font-size: 12px; padding: 4px 14px; cursor: pointer; display: flex; align-items: center; gap: 6px;",
                        disabled: is_loading,
                        onclick: load_older,
                        if is_loading {
                            span { "Loading older messages..." }
                        } else {
                            span { "↑ Load older messages" }
                        }
                    }
                }
            }
            if messages.is_empty() {
                div {
                    style: "margin: auto; text-align: center; color: #8696a0; font-size: 14px; background: rgba(17,27,33,0.8); padding: 10px 18px; border-radius: 8px; max-width: 480px;",
                    "🔒 Messages are end-to-end encrypted. No one outside of this chat, not even WhatsApp, can read or listen to them."
                }
            } else {
                for msg in messages {
                    MessageBubble { key: "{msg.id}", message: msg }
                }
            }
        }
    }
}
