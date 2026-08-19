use dioxus::prelude::*;
use crate::state::AppState;
use crate::ui::chat_list_item::ChatListItem;
use crate::ui::connection_status::ConnectionStatus;

#[component]
pub fn ChatList() -> Element {
    let mut app_state = use_context::<AppState>();
    let chats = app_state.chats.read();
    let mut search_query = use_signal(String::new);

    let filtered_chats: Vec<_> = chats.iter().filter(|c| {
        let q = search_query.read().to_lowercase();
        if q.is_empty() {
            true
        } else {
            c.name.as_deref().unwrap_or("").to_lowercase().contains(&q)
                || c.jid.to_lowercase().contains(&q)
        }
    }).cloned().collect();

    rsx! {
        div {
            style: "width: 30%; min-width: 320px; max-width: 450px; height: 100%; border-right: 1px solid #222e35; background-color: #111b21; display: flex; flex-direction: column;",
            // Header Bar
            div {
                style: "height: 60px; padding: 10px 16px; background-color: #202c33; display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid #222e35;",
                h2 { style: "font-size: 20px; font-weight: 600; color: #e9edef; margin: 0;", "Chats" }
                div {
                    style: "display: flex; align-items: center; gap: 8px;",
                    ConnectionStatus {}
                    button {
                        style: "background: none; border: none; font-size: 18px; cursor: pointer; color: #aebac1; padding: 6px; border-radius: 50%; display: flex; align-items: center; justify-content: center;",
                        onclick: move |_| {
                            *app_state.active_modal.write() = Some("settings".to_string());
                        },
                        "⚙"
                    }
                }
            }
            // Search Bar
            div {
                style: "padding: 8px 12px; background-color: #111b21;",
                input {
                    style: "width: 100%; height: 36px; background-color: #202c33; border: none; border-radius: 8px; padding: 0 12px; color: #e9edef; font-size: 14px; outline: none; box-sizing: border-box;",
                    placeholder: "Search or start new chat",
                    value: "{search_query}",
                    oninput: move |e| search_query.set(e.value())
                }
            }
            // Chat List Items Container
            div {
                style: "flex: 1; overflow-y: auto;",
                if filtered_chats.is_empty() {
                    div {
                        style: "padding: 24px; text-align: center; color: #8696a0; font-size: 14px;",
                        "No chats found"
                    }
                } else {
                    for chat in filtered_chats {
                        ChatListItem { key: "{chat.jid}", chat: chat }
                    }
                }
            }
        }
    }
}
