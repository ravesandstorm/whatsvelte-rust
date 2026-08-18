use dioxus::prelude::*;
use crate::state::AppState;
use crate::ui::message_list::MessageList;
use crate::ui::message_composer::MessageComposer;

#[component]
pub fn Conversation() -> Element {
    let app_state = use_context::<AppState>();
    let active_jid = app_state.active_chat_jid.read();
    let chats = app_state.chats.read();
    let messages_map = app_state.messages.read();

    if let Some(jid) = active_jid.as_ref() {
        let active_chat = chats.iter().find(|c| c.jid == *jid);
        let chat_name = active_chat.and_then(|c| c.name.as_deref()).unwrap_or(jid);
        let message_history = messages_map.get(jid).cloned().unwrap_or_default();

        rsx! {
            div {
                style: "flex: 1; height: 100%; display: flex; flex-direction: column; background-color: #0b141a;",
                // Header
                div {
                    style: "height: 60px; padding: 10px 16px; background-color: #202c33; display: flex; align-items: center; justify-content: space-between; border-bottom: 1px solid #222e35;",
                    div {
                        style: "display: flex; align-items: center; gap: 12px;",
                        div {
                            style: "width: 40px; height: 40px; border-radius: 50%; background-color: #6b7c85; display: flex; align-items: center; justify-content: center; font-weight: 600; color: #ffffff;",
                            "{chat_name.chars().next().unwrap_or('?')}"
                        }
                        div {
                            span { style: "font-size: 16px; font-weight: 500; color: #e9edef;", "{chat_name}" }
                        }
                    }
                }
                // Message List Area
                MessageList { messages: message_history }
                // Message Input Composer
                MessageComposer {}
            }
        }
    } else {
        rsx! {
            div {
                style: "flex: 1; height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; background-color: #222e35; color: #8696a0; border-bottom: 6px solid #00a884;",
                h1 { style: "font-size: 32px; font-weight: 300; color: #e9edef; margin-bottom: 12px;", "WhatsApp Dioxus" }
                p { style: "font-size: 14px; color: #8696a0; max-width: 460px; text-align: center; line-height: 20px;",
                    "Send and receive messages without keeping your phone online. Select a chat from the left sidebar to start messaging."
                }
            }
        }
    }
}
