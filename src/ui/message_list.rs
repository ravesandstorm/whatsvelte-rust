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

    // 1. Initial bottom pinning when chat opens or changes
    let chat_jid_key = chat_jid.clone();
    use_effect(use_reactive(&chat_jid_key, move |_| {
        let _ = document::eval(
            r#"
            (function() {
                var el = document.getElementById('message-list-scroll');
                if (!el) return;
                el.dataset.ready = 'false';
                el.dataset.loading = 'false';
                
                function pinToBottom() {
                    if (el) {
                        el.scrollTop = el.scrollHeight;
                        el.dataset.prevHeight = el.scrollHeight;
                    }
                }
                pinToBottom();
                requestAnimationFrame(pinToBottom);
                setTimeout(pinToBottom, 30);
                setTimeout(pinToBottom, 100);
                setTimeout(function() {
                    pinToBottom();
                    if (el) {
                        el.dataset.ready = 'true';
                    }
                }, 300);
            })();
            "#
        );
    }));

    // 2. Reposition scroll when older messages arrive or smooth scroll down for new live messages
    use_effect(use_reactive(&msg_count, move |_| {
        let _ = document::eval(
            r#"
            (function() {
                var el = document.getElementById('message-list-scroll');
                if (!el) return;
                var prevHeight = parseInt(el.dataset.prevHeight || '0', 10);
                var newHeight = el.scrollHeight;
                el.dataset.prevHeight = newHeight;
                
                if (el.dataset.loading === 'true') {
                    // Reposition scroll to the exact position before prepend (+30px out of top trigger)
                    var diff = newHeight - prevHeight;
                    if (diff > 0) {
                        el.scrollTop = diff + 30;
                    }
                    el.dataset.loading = 'false';
                    setTimeout(function() {
                        if (el) el.dataset.ready = 'true';
                    }, 250);
                } else if (el.dataset.ready !== 'true') {
                    el.scrollTop = el.scrollHeight;
                }
            })();
            "#
        );
    }));

    // 3. Listen exclusively for the top-scroll event emitted from JavaScript
    let chat_jid_for_stream = chat_jid.clone();
    use_future(move || {
        let chat_jid = chat_jid_for_stream.clone();
        let mut app_state = app_state;
        async move {
            let js = r#"
                var el = document.getElementById('message-list-scroll');
                if (!el) return;
                
                if (el._topScrollHandler) {
                    el.removeEventListener('scroll', el._topScrollHandler);
                }
                
                el._topScrollHandler = function() {
                    if (el.dataset.ready !== 'true' || el.dataset.loading === 'true') return;
                    // Trigger ONLY when user is at the very top (scrollTop <= 5px) and container is scrollable
                    if (el.scrollTop <= 5 && el.scrollHeight > el.clientHeight + 80) {
                        el.dataset.loading = 'true';
                        el.dataset.ready = 'false';
                        el.dataset.prevHeight = el.scrollHeight;
                        dioxus.send("load_older");
                    }
                };
                
                el.addEventListener('scroll', el._topScrollHandler, { passive: true });
            "#;
            
            let mut eval = document::eval(js);
            while let Ok(msg) = eval.recv::<String>().await {
                if msg == "load_older" {
                    let has_more_val = *app_state.has_more_messages.read();
                    let is_loading_val = *app_state.is_loading_messages.read();
                    if !has_more_val || is_loading_val {
                        let _ = document::eval("var el = document.getElementById('message-list-scroll'); if (el) { el.dataset.loading = 'false'; el.dataset.ready = 'true'; }");
                        continue;
                    }
                    
                    let oldest_ts = app_state.messages.read().get(&chat_jid)
                        .and_then(|msgs| msgs.first())
                        .map(|m| m.timestamp)
                        .unwrap_or(0);
                    
                    if oldest_ts <= 0 {
                        let _ = document::eval("var el = document.getElementById('message-list-scroll'); if (el) { el.dataset.loading = 'false'; el.dataset.ready = 'true'; }");
                        continue;
                    }
                    
                    *app_state.is_loading_messages.write() = true;
                    let fetch_js = format!("return await window.whatsappDb.loadChatMessages({:?}, 50, {});", chat_jid, oldest_ts);
                    if let Ok(eval_val) = document::eval(&fetch_js).await {
                        if let Some(json_str) = eval_val.as_str() {
                            if let Ok(resp) = serde_json::from_str::<MessagesResponse>(json_str) {
                                if !resp.messages.is_empty() {
                                    let mut map = app_state.messages.write();
                                    if let Some(current) = map.get_mut(&chat_jid) {
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
                }
            }
        }
    });

    rsx! {
        div {
            id: "message-list-scroll",
            style: "flex: 1 1 0; min-height: 0; height: 0; overflow-y: auto; overflow-x: hidden; padding: 20px 32px; display: flex; flex-direction: column; gap: 6px; background-color: #0b141a; background-image: radial-gradient(#1f2c34 1px, transparent 0); background-size: 20px 20px; box-sizing: border-box;",
            if is_loading && has_more {
                div {
                    style: "display: flex; justify-content: center; padding: 6px 0; font-size: 12px; color: #53bdeb;",
                    span { "Loading older messages..." }
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
