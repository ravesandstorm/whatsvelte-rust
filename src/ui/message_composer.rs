use std::sync::Arc;
use dioxus::prelude::*;
use crate::state::AppState;
use crate::session::SessionManager;
use crate::dto::MessageDto;

#[component]
pub fn MessageComposer() -> Element {
    let mut app_state = use_context::<AppState>();
    let session_mgr = use_context::<Arc<SessionManager>>();
    let mut text = use_signal(String::new);

    let mut send_message = {
        let mgr = session_mgr.clone();
        move || {
            let content = text.read().trim().to_string();
            if content.is_empty() {
                return;
            }

            if let Some(active_jid) = app_state.active_chat_jid.read().as_ref() {
                let jid_key = active_jid.clone();
                let now_secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                let temp_id = format!("msg_{}", now_secs);

                let new_msg = MessageDto {
                    id: temp_id.clone(),
                    chat_jid: jid_key.clone(),
                    sender_jid: "me".to_string(),
                    from_me: true,
                    timestamp: now_secs,
                    push_name: Some("Me".to_string()),
                    text: Some(content.clone()),
                    kind: "text".to_string(),
                    thumbnail: None,
                    media: None,
                    quoted: None,
                };

                // Optimistic UI update
                app_state.messages.write().entry(jid_key.clone()).or_default().push(new_msg);

                // Update chat preview in list
                let mut chats = app_state.chats.write();
                if let Some(c) = chats.iter_mut().find(|c| c.jid == jid_key) {
                    c.last_message = Some(content.clone());
                    c.timestamp = now_secs;
                }

                // Send via backend
                let mgr_clone = mgr.clone();
                let jid_to_send = jid_key.clone();
                let text_to_send = content.clone();
                spawn(async move {
                    if let Err(e) = crate::commands::messaging::send_text(
                        jid_to_send,
                        text_to_send,
                        None,
                        &mgr_clone,
                    ).await {
                        log::error!("Failed to send message: {e}");
                    }
                });
            }

            text.set(String::new());
        }
    };

    rsx! {
        div {
            style: "min-height: 62px; background-color: #202c33; padding: 8px 16px; display: flex; align-items: center; gap: 12px; border-top: 1px solid #222e35; box-sizing: border-box;",
            button {
                style: "background: none; border: none; font-size: 20px; cursor: pointer; color: #8696a0;",
                "😊"
            }
            button {
                style: "background: none; border: none; font-size: 20px; cursor: pointer; color: #8696a0;",
                "📎"
            }
            input {
                style: "flex: 1; height: 42px; background-color: #2a3942; border: none; border-radius: 8px; padding: 0 16px; color: #e9edef; font-size: 15px; outline: none; box-sizing: border-box;",
                placeholder: "Type a message",
                value: "{text}",
                oninput: move |e| text.set(e.value()),
                onkeydown: {
                    let mut send_fn = send_message.clone();
                    move |e| {
                        if e.key() == Key::Enter {
                            send_fn();
                        }
                    }
                }
            }
            button {
                style: "background-color: #00a884; border: none; width: 42px; height: 42px; border-radius: 50%; display: flex; align-items: center; justify-content: center; cursor: pointer; color: #111b21; font-weight: bold; flex-shrink: 0;",
                onclick: move |_| {
                    send_message();
                },
                "➤"
            }
        }
    }
}
