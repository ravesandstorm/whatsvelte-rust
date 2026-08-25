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
    let has_min_messages = msg_count >= 50;

    // 1. When chat_jid changes, force immediate scroll to bottom
    let chat_jid_key = chat_jid.clone();
    use_effect(use_reactive(&chat_jid_key, move |_| {
        let _ = document::eval(
            r#"
            (function() {
                var el = document.getElementById('message-list-scroll');
                if (el) {
                    el.scrollTop = el.scrollHeight;
                    el.dataset.prevHeight = el.scrollHeight;
                    el.dataset.isLoadingOlder = 'false';
                    el.dataset.initialLoad = 'true';
                    setTimeout(function() {
                        if (el) {
                            el.scrollTop = el.scrollHeight;
                            el.dataset.initialLoad = 'false';
                        }
                    }, 50);
                }
            })();
            "#
        );
    }));

    // 2. When message count changes (older prepended or new appended)
    use_effect(use_reactive(&msg_count, move |_| {
        let _ = document::eval(
            r#"
            (function() {
                var el = document.getElementById('message-list-scroll');
                if (el) {
                    var prevHeight = parseInt(el.dataset.prevHeight || '0', 10);
                    var newHeight = el.scrollHeight;
                    el.dataset.prevHeight = newHeight;
                    
                    if (el.dataset.isLoadingOlder === 'true') {
                        // Preserve scroll position when older 50 messages are loaded
                        var diff = newHeight - prevHeight;
                        if (diff > 0) {
                            el.scrollTop = diff;
                        }
                        el.dataset.isLoadingOlder = 'false';
                    } else if (el.dataset.initialLoad === 'true') {
                        el.scrollTop = el.scrollHeight;
                    } else {
                        // Smooth scroll to bottom for incoming live messages if near bottom
                        var distanceFromBottom = el.scrollHeight - el.scrollTop - el.clientHeight;
                        if (distanceFromBottom < 200) {
                            el.scrollTo({ top: el.scrollHeight, behavior: 'smooth' });
                        }
                    }
                }
            })();
            "#
        );
    }));

    let load_older = {
        let chat_jid = chat_jid.clone();
        let oldest_ts = messages.first().map(|m| m.timestamp).unwrap_or(0);
        move || {
            if is_loading || !has_more || oldest_ts <= 0 {
                return;
            }
            let jid = chat_jid.clone();
            let mut app_state = app_state;
            spawn(async move {
                *app_state.is_loading_messages.write() = true;
                let _ = document::eval(
                    r#"
                    var el = document.getElementById('message-list-scroll');
                    if (el) {
                        el.dataset.isLoadingOlder = 'true';
                        el.dataset.prevHeight = el.scrollHeight;
                    }
                    "#
                );
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

    let load_older_on_scroll = load_older.clone();

    rsx! {
        div {
            id: "message-list-scroll",
            style: "flex: 1 1 0; min-height: 0; height: 0; overflow-y: auto; overflow-x: hidden; scroll-behavior: smooth; padding: 20px 32px; display: flex; flex-direction: column; gap: 6px; background-color: #0b141a; background-image: radial-gradient(#1f2c34 1px, transparent 0); background-size: 20px 20px; box-sizing: border-box;",
            onscroll: move |_| {
                let _ = document::eval(
                    r#"
                    var el = document.getElementById('message-list-scroll');
                    if (el && el.dataset.initialLoad !== 'true' && el.dataset.isLoadingOlder !== 'true') {
                        if (el.scrollTop <= 20 && el.scrollHeight > el.clientHeight + 50) {
                            return true;
                        }
                    }
                    return false;
                    "#
                );
                // Trigger auto load older when user scrolls to top
                if has_more && !is_loading && has_min_messages {
                    load_older_on_scroll();
                }
            },
            if has_more {
                div {
                    style: "display: flex; justify-content: center; padding: 8px 0;",
                    button {
                        style: "background-color: #202c33; border: 1px solid #374248; border-radius: 16px; color: #53bdeb; font-size: 12px; padding: 4px 14px; cursor: pointer; display: flex; align-items: center; gap: 6px; transition: background-color 0.15s ease;",
                        disabled: is_loading,
                        onclick: move |_| load_older(),
                        if is_loading {
                            span { "Loading older messages..." }
                        } else {
                            span { "↑ Load older messages (50 more)" }
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
