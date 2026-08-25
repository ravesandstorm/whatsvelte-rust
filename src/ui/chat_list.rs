use dioxus::prelude::*;
use crate::state::AppState;
use crate::ui::chat_list_item::ChatListItem;
use crate::ui::connection_status::ConnectionStatus;
use crate::ui::jid_utils::resolve_display_name;

#[component]
pub fn ChatList() -> Element {
    let mut app_state = use_context::<AppState>();
    let chats = app_state.chats.read();
    let sync_progress = app_state.sync_progress.read();
    let contacts = app_state.contacts.read();
    let mut search_query = use_signal(String::new);

    let sync_pct = sync_progress.as_ref().map(|p| {
        if p.total > 0 {
            ((p.loaded as f32 / p.total as f32) * 100.0).min(100.0) as u32
        } else {
            0
        }
    }).unwrap_or(0);

    let filtered_chats: Vec<_> = chats.iter().filter(|c| {
        let q = search_query.read().to_lowercase();
        if q.is_empty() {
            true
        } else {
            let contact_name = contacts.get(&c.jid).and_then(|contact| contact.verified_name.as_deref().or(contact.name.as_deref()));
            let resolved = resolve_display_name(&c.jid, contact_name.or(c.name.as_deref()), None);
            resolved.to_lowercase().contains(&q)
                || c.jid.to_lowercase().contains(&q)
        }
    }).cloned().collect();

    rsx! {
        div {
            style: "width: 30%; min-width: 320px; max-width: 450px; height: 100%; min-height: 0; border-right: 1px solid #222e35; background-color: #111b21; display: flex; flex-direction: column; overflow: hidden; flex-shrink: 0;",
            // Header Bar
            div {
                style: "height: 60px; min-height: 60px; padding: 10px 16px; background-color: #202c33; display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid #222e35; flex-shrink: 0; box-sizing: border-box;",
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
            // Syncing Status Banner (if background sync is active)
            if let Some(p) = sync_progress.as_ref() {
                if p.is_syncing {
                    div {
                        style: "background-color: #1f2c34; padding: 6px 12px; border-bottom: 1px solid #222e35; display: flex; flex-direction: column; gap: 4px; flex-shrink: 0;",
                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; font-size: 11.5px; color: #53bdeb;",
                            span { "🔄 {p.title}" }
                            if p.total > 0 {
                                span { "{p.loaded}/{p.total} ({sync_pct}%)" }
                            }
                        }
                        if p.total > 0 {
                            div {
                                style: "width: 100%; height: 3px; background-color: #111b21; border-radius: 2px; overflow: hidden;",
                                div {
                                    style: "height: 100%; width: {sync_pct}%; background-color: #00a884; transition: width 0.2s ease;",
                                }
                            }
                        }
                    }
                }
            }
            // Search Bar
            div {
                style: "padding: 8px 12px; background-color: #111b21; flex-shrink: 0;",
                input {
                    style: "width: 100%; height: 36px; background-color: #202c33; border: none; border-radius: 8px; padding: 0 12px; color: #e9edef; font-size: 14px; outline: none; box-sizing: border-box;",
                    placeholder: "Search or start new chat",
                    value: "{search_query}",
                    oninput: move |e| search_query.set(e.value())
                }
            }
            // Chat List Items Container (Scrollable)
            div {
                id: "chat-list-scroll",
                style: "flex: 1 1 0; min-height: 0; height: 0; overflow-y: auto; overflow-x: hidden; scroll-behavior: smooth;",
                if filtered_chats.is_empty() {
                    div {
                        style: "padding: 32px 16px; text-align: center; color: #8696a0; font-size: 14px;",
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
