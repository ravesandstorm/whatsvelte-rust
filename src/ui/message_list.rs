use std::collections::HashSet;
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

    // 1. Instant bottom snap on chat open/switch
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
                    el.dataset.hasScrolledDown = 'false';
                    // Re-assert bottom after sub-renders
                    setTimeout(function() {
                        if (el) {
                            el.scrollTop = el.scrollHeight;
                        }
                    }, 30);
                    setTimeout(function() {
                        if (el) {
                            el.scrollTop = el.scrollHeight;
                        }
                    }, 100);
                }
            })();
            "#
        );
    }));

    // 2. Adjust scroll position when older messages are loaded or auto-scroll for new live message
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
                        // Keep viewport anchored to exact previous scroll point
                        var diff = newHeight - prevHeight;
                        if (diff > 0) {
                            el.scrollTop = diff;
                        }
                        el.dataset.isLoadingOlder = 'false';
                    } else {
                        var distanceFromBottom = el.scrollHeight - el.scrollTop - el.clientHeight;
                        if (distanceFromBottom < 150) {
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
                                    let mut combined = resp.messages;
                                    combined.append(current);
                                    let mut seen = HashSet::new();
                                    let mut deduplicated = Vec::new();
                                    for m in combined {
                                        if m.id.is_empty() || seen.insert(m.id.clone()) {
                                            deduplicated.push(m);
                                        }
                                    }
                                    deduplicated.sort_by_key(|m| m.timestamp);
                                    *current = deduplicated;
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
            style: "flex: 1 1 0; min-height: 0; height: 0; overflow-y: auto; overflow-x: hidden; padding: 20px 32px; display: flex; flex-direction: column; gap: 6px; background-color: #0b141a; background-image: radial-gradient(#1f2c34 1px, transparent 0); background-size: 20px 20px; box-sizing: border-box;",
            onscroll: move |_| {
                // Track user scroll state
                let _ = document::eval(
                    r#"
                    var el = document.getElementById('message-list-scroll');
                    if (el) {
                        var distanceFromBottom = el.scrollHeight - el.scrollTop - el.clientHeight;
                        if (distanceFromBottom > 200) {
                            el.dataset.hasScrolledDown = 'true';
                        }
                    }
                    "#
                );
            },
            if has_more {
                div {
                    style: "display: flex; justify-content: center; padding: 10px 0; margin-bottom: 6px;",
                    button {
                        style: "background-color: #202c33; border: 1px solid #374248; border-radius: 18px; color: #53bdeb; font-size: 12.5px; font-weight: 500; padding: 6px 18px; cursor: pointer; display: flex; align-items: center; gap: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.3); transition: all 0.15s ease;",
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
                for (idx, msg) in messages.iter().enumerate() {
                    {
                        let unique_key = if msg.id.is_empty() {
                            format!("msg_{}_{}_{}", msg.chat_jid, msg.timestamp, idx)
                        } else {
                            format!("msg_{}_{}_{}", msg.id, msg.timestamp, idx)
                        };
                        rsx! {
                            MessageBubble { key: "{unique_key}", message: msg.clone() }
                        }
                    }
                }
            }
        }
    }
}
