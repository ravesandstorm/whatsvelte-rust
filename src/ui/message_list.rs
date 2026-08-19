use dioxus::prelude::*;
use crate::dto::MessageDto;
use crate::ui::message_bubble::MessageBubble;

#[component]
pub fn MessageList(messages: Vec<MessageDto>) -> Element {
    let msg_count = messages.len();
    use_effect(use_reactive(&msg_count, move |_| {
        let _ = document::eval(
            r#"
            (function() {
                var el = document.getElementById('message-list-scroll');
                if (el) {
                    el.scrollTop = el.scrollHeight;
                }
            })();
            "#
        );
    }));

    rsx! {
        div {
            id: "message-list-scroll",
            style: "flex: 1 1 0; min-height: 0; height: 0; overflow-y: auto; overflow-x: hidden; padding: 20px 32px; display: flex; flex-direction: column; gap: 6px; background-color: #0b141a; background-image: radial-gradient(#1f2c34 1px, transparent 0); background-size: 20px 20px; box-sizing: border-box;",
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
