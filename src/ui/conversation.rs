use dioxus::prelude::*;
use crate::state::AppState;
use crate::ui::avatar::Avatar;
use crate::ui::jid_utils::resolve_display_name;
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
        let message_history = messages_map.get(jid).cloned().unwrap_or_default();
        let fallback_push = message_history.iter().find(|m| !m.from_me).and_then(|m| m.push_name.as_deref());
        let display_name = resolve_display_name(jid, active_chat.and_then(|c| c.name.as_deref()), fallback_push);

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
                }
                // Message List Area (Scrollable flex item)
                MessageList { messages: message_history }
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
