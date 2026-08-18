use std::collections::HashMap;
use dioxus::prelude::*;
use crate::dto::{ChatDto, ContactDto, MessageDto, StatusDto};

#[derive(Clone, Copy)]
pub struct AppState {
    pub session: Signal<StatusDto>,
    pub qr_code: Signal<Option<String>>,
    pub pair_code: Signal<Option<String>>,
    pub chats: Signal<Vec<ChatDto>>,
    pub active_chat_jid: Signal<Option<String>>,
    pub messages: Signal<HashMap<String, Vec<MessageDto>>>,
    pub contacts: Signal<HashMap<String, ContactDto>>,
    pub dark_theme: Signal<bool>,
    pub active_modal: Signal<Option<String>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            session: Signal::new(StatusDto {
                session_id: "default".to_string(),
                logged_in: false,
                connected: false,
                jid: None,
                registered: false,
                push_name: None,
            }),
            qr_code: Signal::new(None),
            pair_code: Signal::new(None),
            chats: Signal::new(Vec::new()),
            active_chat_jid: Signal::new(None),
            messages: Signal::new(HashMap::new()),
            contacts: Signal::new(HashMap::new()),
            dark_theme: Signal::new(true),
            active_modal: Signal::new(None),
        }
    }
}
