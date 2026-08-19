use std::sync::Arc;
use std::collections::HashMap;
use dioxus::prelude::*;
use crate::state::AppState;
use crate::session::SessionManager;
use crate::dto::{ChatDto, StatusDto, UiEvent};
use crate::ui::main_layout::MainLayout;
use crate::ui::pairing_screen::PairingScreen;
use crate::ui::loading_screen::LoadingScreen;

#[component]
pub fn App() -> Element {
    let app_state = use_context_provider(AppState::new);
    let session_mgr = use_context_provider(|| {
        let base_dir = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("whatdioxus-rust");
        Arc::new(SessionManager::new(base_dir))
    });

    let session = app_state.session.read();

    use_future(move || {
        let mgr = session_mgr.clone();
        let mut app_state = app_state;
        async move {
            // Check initial auth status and boot session if needed
            match crate::commands::auth::auth_status(None, &mgr).await {
                Ok(status) => {
                    *app_state.session.write() = status;
                }
                Err(e) => {
                    log::error!("Initial auth status error: {e}");
                }
            }

            let rx = mgr.event_receiver();
            while let Ok((_sid, event)) = rx.recv().await {
                match event {
                    UiEvent::QrCode(code) => {
                        *app_state.qr_code.write() = Some(code);
                    }
                    UiEvent::PairCode(code) => {
                        *app_state.pair_code.write() = Some(code);
                    }
                    UiEvent::Connected => {
                        app_state.session.write().connected = true;
                    }
                    UiEvent::LoggedIn => {
                        {
                            let mut s = app_state.session.write();
                            s.logged_in = true;
                            s.connected = true;
                            s.registered = true;
                        }
                        *app_state.qr_code.write() = None;
                        *app_state.pair_code.write() = None;
                    }

                    UiEvent::Disconnected => {
                        app_state.session.write().connected = false;
                    }
                    UiEvent::LoggedOut => {
                        *app_state.session.write() = StatusDto {
                            session_id: "default".to_string(),
                            logged_in: false,
                            connected: false,
                            jid: None,
                            registered: false,
                            push_name: None,
                        };
                        *app_state.chats.write() = Vec::new();
                        *app_state.messages.write() = HashMap::new();
                    }
                    UiEvent::PushName(name) => {
                        app_state.session.write().push_name = Some(name);
                    }
                    UiEvent::NewMessage(msg) => {
                        let chat_jid = msg.chat_jid.clone();
                        let text_preview = msg.text.clone();
                        let ts = msg.timestamp;

                        app_state.messages.write().entry(chat_jid.clone()).or_default().push(msg);

                        let mut chats = app_state.chats.write();
                        if let Some(existing) = chats.iter_mut().find(|c| c.jid == chat_jid) {
                            existing.last_message = text_preview;
                            existing.timestamp = ts;
                        } else {
                            chats.insert(0, ChatDto {
                                jid: chat_jid,
                                name: None,
                                last_message: text_preview,
                                timestamp: ts,
                                unread: 0,
                            });
                        }
                    }
                    UiEvent::MessageUpdate(update) => {
                        let mut messages_map = app_state.messages.write();
                        if let Some(msgs) = messages_map.get_mut(&update.chat_jid) {
                            if update.kind == "revoke" {
                                msgs.retain(|m| m.id != update.target_id);
                            } else if update.kind == "edit" {
                                if let Some(m) = msgs.iter_mut().find(|m| m.id == update.target_id) {
                                    m.text = update.text;
                                }
                            }
                        }
                    }
                    UiEvent::HistorySync(history) => {
                        let mut chats_map = HashMap::new();
                        for c in app_state.chats.read().iter() {
                            chats_map.insert(c.jid.clone(), c.clone());
                        }
                        let mut msgs_map = app_state.messages.write();
                        for h in history {
                            for c in h.chats {
                                chats_map.insert(c.jid.clone(), c);
                            }
                            for m in h.messages {
                                msgs_map.entry(m.chat_jid.clone()).or_default().push(m);
                            }
                        }
                        for msgs in msgs_map.values_mut() {
                            msgs.sort_by_key(|m| m.timestamp);
                            msgs.dedup_by(|a, b| a.id == b.id);
                        }
                        let mut new_chats: Vec<_> = chats_map.into_values().collect();
                        new_chats.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                        *app_state.chats.write() = new_chats;
                    }
                    UiEvent::ChatFlags(_) | UiEvent::Receipt(_) => {}
                }
            }
        }
    });

    rsx! {
        if session.logged_in {
            MainLayout {}
        } else if session.registered {
            LoadingScreen {}
        } else {
            PairingScreen {}
        }
    }
}
