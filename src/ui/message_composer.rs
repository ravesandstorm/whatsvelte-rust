use dioxus::prelude::*;
use crate::state::AppState;
use crate::dto::MessageDto;

#[component]
pub fn MessageComposer() -> Element {
    let mut app_state = use_context::<AppState>();
    let mut text = use_signal(String::new);

    rsx! {
        div {
            style: "min-height: 62px; background-color: #202c33; padding: 8px 16px; display: flex; align-items: center; gap: 12px; border-top: 1px solid #222e35;",
            button {
                style: "background: none; border: none; font-size: 20px; cursor: pointer; color: #8696a0;",
                "😊"
            }
            button {
                style: "background: none; border: none; font-size: 20px; cursor: pointer; color: #8696a0;",
                "📎"
            }
            input {
                style: "flex: 1; height: 42px; background-color: #2a3942; border: none; border-radius: 8px; padding: 0 16px; color: #e9edef; font-size: 15px; outline: none;",
                placeholder: "Type a message",
                value: "{text}",
                oninput: move |e| text.set(e.value()),
                onkeydown: move |e| {
                    if e.key() == Key::Enter {
                        let content = text.read().trim().to_string();
                        if !content.is_empty() {
                            if let Some(active_jid) = app_state.active_chat_jid.read().as_ref() {
                                let jid_key = active_jid.clone();
                                let new_msg = MessageDto {
                                    id: format!("msg_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis()),
                                    chat_jid: jid_key.clone(),
                                    sender_jid: "me".to_string(),
                                    from_me: true,
                                    timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64,
                                    push_name: Some("Me".to_string()),
                                    text: Some(content),
                                    kind: "text".to_string(),
                                    thumbnail: None,
                                    media: None,
                                    quoted: None,
                                };
                                app_state.messages.write().entry(jid_key).or_default().push(new_msg);
                            }
                            text.set(String::new());
                        }
                    }
                }
            }
            button {
                style: "background-color: #00a884; border: none; width: 42px; height: 42px; border-radius: 50%; display: flex; align-items: center; justify-content: center; cursor: pointer; color: #111b21; font-weight: bold;",
                onclick: move |_| {
                    let content = text.read().trim().to_string();
                    if !content.is_empty() {
                        if let Some(active_jid) = app_state.active_chat_jid.read().as_ref() {
                            let jid_key = active_jid.clone();
                            let new_msg = MessageDto {
                                id: format!("msg_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis()),
                                chat_jid: jid_key.clone(),
                                sender_jid: "me".to_string(),
                                from_me: true,
                                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64,
                                push_name: Some("Me".to_string()),
                                text: Some(content),
                                kind: "text".to_string(),
                                thumbnail: None,
                                media: None,
                                quoted: None,
                            };
                            app_state.messages.write().entry(jid_key).or_default().push(new_msg);
                        }
                        text.set(String::new());
                    }
                },
                "➤"
            }
        }
    }
}
