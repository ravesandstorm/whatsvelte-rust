use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;
use dioxus::prelude::*;
use serde::Deserialize;
use crate::state::AppState;
use crate::session::SessionManager;
use crate::dto::{ChatDto, ContactDto, StatusDto, UiEvent};
use crate::ui::main_layout::MainLayout;
use crate::ui::pairing_screen::PairingScreen;
use crate::ui::loading_screen::LoadingScreen;

#[derive(Deserialize)]
struct CachedData {
    #[serde(default)]
    chats: Vec<ChatDto>,
    #[serde(default)]
    contacts: Vec<ContactDto>,
}

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
    let has_chats = !app_state.chats.read().is_empty();

    use_future(move || {
        let mgr = session_mgr.clone();
        let mut app_state = app_state;
        async move {
            // 1. Hydrate chats and contacts from IndexedDB cache immediately on startup (keep messages unloaded)
            if let Ok(eval_val) = document::eval("return await window.whatsappDb.loadAll();").await {
                if let Some(json_str) = eval_val.as_str() {
                    if let Ok(cached) = serde_json::from_str::<CachedData>(json_str) {
                        log::info!(
                            "Hydrated from IndexedDB: {} chats, {} contacts",
                            cached.chats.len(),
                            cached.contacts.len()
                        );

                        if !cached.contacts.is_empty() {
                            let mut contacts_map = HashMap::new();
                            for c in cached.contacts {
                                contacts_map.insert(c.jid.clone(), c);
                            }
                            *app_state.contacts.write() = contacts_map;
                        }

                        if !cached.chats.is_empty() {
                            let mut sorted_chats = cached.chats;
                            sorted_chats.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                            *app_state.chats.write() = sorted_chats;
                        }
                    }
                }
            }

            // 2. Check initial auth status and boot session if needed
            match crate::commands::auth::auth_status(None, &mgr).await {
                Ok(status) => {
                    let is_reg = status.registered;
                    *app_state.session.write() = status;

                    // Safety timeout: If registered and chats exist, transition to logged_in smoothly
                    if is_reg {
                        let mut app_state_inner = app_state;
                        spawn(async move {
                            tokio::time::sleep(Duration::from_secs(3)).await;
                            if !app_state_inner.chats.read().is_empty() {
                                app_state_inner.session.write().logged_in = true;
                            }
                        });
                    }
                }
                Err(e) => {
                    log::error!("Initial auth status error: {e}");
                }
            }

            // 3. Process reactive events from backend
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
                        *app_state.contacts.write() = HashMap::new();
                        *app_state.qr_code.write() = None;
                        *app_state.pair_code.write() = None;
                        *app_state.sync_progress.write() = None;

                        // Clear IndexedDB cache
                        let _ = document::eval("window.whatsappDb.clearAll();");

                        let mgr_reset = mgr.clone();
                        spawn(async move {
                            if let Err(e) = mgr_reset.reset("default").await {
                                log::error!("Failed to restart session after logout: {e}");
                            }
                        });
                    }
                    UiEvent::PushName(name) => {
                        app_state.session.write().push_name = Some(name);
                    }
                    UiEvent::SyncProgress(progress) => {
                        let is_done = !progress.is_syncing || (progress.total > 0 && progress.loaded >= progress.total);
                        *app_state.sync_progress.write() = Some(progress.clone());

                        if is_done {
                            app_state.session.write().logged_in = true;
                            let mut app_state_inner = app_state;
                            spawn(async move {
                                tokio::time::sleep(Duration::from_millis(1500)).await;
                                *app_state_inner.sync_progress.write() = None;
                            });
                        }
                    }
                    UiEvent::ContactUpdate(contact) => {
                        let jid = contact.jid.clone();
                        let new_name = contact.verified_name.clone().or_else(|| contact.name.clone());
                        app_state.contacts.write().insert(jid.clone(), contact.clone());

                        // Update chat title if matching
                        if let Some(name_str) = new_name {
                            let mut chats = app_state.chats.write();
                            if let Some(c) = chats.iter_mut().find(|c| c.jid == jid) {
                                if c.name.is_none() || c.name.as_deref() == Some(&jid) {
                                    c.name = Some(name_str);
                                }
                            }
                        }

                        // Persist to IndexedDB
                        if let Ok(contact_json) = serde_json::to_string(&contact) {
                            let js = format!("window.whatsappDb.saveContact({contact_json});");
                            let _ = document::eval(&js);
                        }
                    }
                    UiEvent::PictureUpdate { jid, url } => {
                        let mut contacts = app_state.contacts.write();
                        if let Some(c) = contacts.get_mut(&jid) {
                            c.picture_url = url.clone();
                        } else {
                            contacts.insert(
                                jid.clone(),
                                ContactDto {
                                    jid: jid.clone(),
                                    name: None,
                                    verified_name: None,
                                    lid: None,
                                    picture_url: url.clone(),
                                },
                            );
                        }
                        if let Ok(c_json) = serde_json::to_string(&contacts.get(&jid)) {
                            let _ = document::eval(&format!("window.whatsappDb.saveContact({c_json});"));
                        }
                    }
                    UiEvent::NewMessage(msg) => {
                        let chat_jid = msg.chat_jid.clone();
                        let text_preview = msg.text.clone();
                        let ts = msg.timestamp;
                        let push_name = if !msg.from_me { msg.push_name.clone() } else { None };

                        // Only append to in-memory messages if this is the active chat
                        let is_active = app_state.active_chat_jid.read().as_deref() == Some(&chat_jid);
                        if is_active {
                            app_state.messages.write().entry(chat_jid.clone()).or_default().push(msg.clone());
                        }

                        let updated_chat = {
                            let mut chats = app_state.chats.write();
                            if let Some(existing) = chats.iter_mut().find(|c| c.jid == chat_jid) {
                                existing.last_message = text_preview.clone();
                                existing.timestamp = ts;
                                if existing.name.is_none() && push_name.is_some() {
                                    existing.name = push_name.clone();
                                }
                                existing.clone()
                            } else {
                                let new_chat = ChatDto {
                                    jid: chat_jid,
                                    name: push_name,
                                    last_message: text_preview,
                                    timestamp: ts,
                                    unread: 0,
                                };
                                chats.insert(0, new_chat.clone());
                                new_chat
                            }
                        };

                        // Sort chats by date and time (newest timestamp first)
                        app_state.chats.write().sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

                        // Persist to IndexedDB
                        if let (Ok(msg_json), Ok(chat_json)) = (serde_json::to_string(&msg), serde_json::to_string(&updated_chat)) {
                            let js = format!(
                                "window.whatsappDb.saveMessages([{msg_json}]); window.whatsappDb.saveChats([{chat_json}]);"
                            );
                            let _ = document::eval(&js);
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
                        let mut all_new_chats = Vec::new();
                        let mut all_new_msgs = Vec::new();
                        let active_jid = app_state.active_chat_jid.read().clone();

                        let mut chats_map = HashMap::new();
                        for c in app_state.chats.read().iter() {
                            chats_map.insert(c.jid.clone(), c.clone());
                        }

                        for h in history {
                            for c in h.chats {
                                all_new_chats.push(c.clone());
                                chats_map.entry(c.jid.clone())
                                    .and_modify(|existing: &mut ChatDto| {
                                        if existing.name.is_none() && c.name.is_some() {
                                            existing.name = c.name.clone();
                                        }
                                        if c.timestamp >= existing.timestamp {
                                            existing.last_message = c.last_message.clone();
                                            existing.timestamp = c.timestamp;
                                        }
                                    })
                                    .or_insert(c);
                            }
                            for m in h.messages {
                                // Only keep in memory if it belongs to currently open active chat
                                if let Some(active) = active_jid.as_ref() {
                                    if &m.chat_jid == active {
                                        app_state.messages.write().entry(m.chat_jid.clone()).or_default().push(m.clone());
                                    }
                                }
                                all_new_msgs.push(m);
                            }
                        }

                        let mut new_chats: Vec<_> = chats_map.into_values().collect();
                        new_chats.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                        *app_state.chats.write() = new_chats.clone();

                        // Update sync progress
                        let added_count = all_new_msgs.len() as u32;
                        if let Some(prog) = app_state.sync_progress.write().as_mut() {
                            prog.loaded = (prog.loaded + added_count).min(prog.total.max(prog.loaded + added_count));
                        }

                        // Persist to IndexedDB
                        if let (Ok(chats_json), Ok(msgs_json)) = (serde_json::to_string(&new_chats), serde_json::to_string(&all_new_msgs)) {
                            let js = format!(
                                "window.whatsappDb.saveChats({chats_json}); window.whatsappDb.saveMessages({msgs_json});"
                            );
                            let _ = document::eval(&js);
                        }
                    }
                    UiEvent::Receipt(receipt) => {
                        let mut messages_map = app_state.messages.write();
                        if let Some(msgs) = messages_map.get_mut(&receipt.chat_jid) {
                            for msg in msgs.iter_mut() {
                                if receipt.message_ids.contains(&msg.id) {
                                    msg.status = Some(receipt.status.clone());
                                }
                            }
                        }
                    }
                    UiEvent::ChatFlags(_) => {}
                }
            }
        }
    });

    rsx! {
        if session.logged_in || (session.registered && has_chats) {
            MainLayout {}
        } else if session.registered {
            LoadingScreen {}
        } else {
            PairingScreen {}
        }
    }
}
