use std::sync::Arc;
use dioxus::prelude::*;
use crate::state::AppState;
use crate::session::SessionManager;
use crate::dto::StatusDto;

#[component]
pub fn SettingsPanel() -> Element {
    let mut app_state = use_context::<AppState>();
    let session_mgr = use_context::<Arc<SessionManager>>();
    let session = app_state.session.read();

    let close_modal = move |_| {
        *app_state.active_modal.write() = None;
    };

    let logout = {
        let mgr = session_mgr.clone();
        move |_| {
            let mgr_clone = mgr.clone();
            *app_state.session.write() = StatusDto {
                session_id: "default".to_string(),
                logged_in: false,
                connected: false,
                jid: None,
                registered: false,
                push_name: None,
            };
            *app_state.chats.write() = Vec::new();
            *app_state.messages.write() = std::collections::HashMap::new();
            *app_state.active_modal.write() = None;

            spawn(async move {
                let _ = crate::commands::auth::auth_logout(None, &mgr_clone).await;
            });
        }
    };

    rsx! {
        div {
            style: "position: fixed; inset: 0; background: rgba(0,0,0,0.6); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            div {
                style: "width: 450px; max-width: 90vw; background: #202c33; border-radius: 12px; padding: 24px; color: #e9edef; display: flex; flex-direction: column; gap: 16px; box-shadow: 0 8px 32px rgba(0,0,0,0.4);",
                header {
                    style: "display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid #222e35; padding-bottom: 12px;",
                    h2 { style: "margin: 0; font-size: 18px; color: #00a884;", "Settings" }
                    button {
                        style: "background: none; border: none; color: #8696a0; font-size: 20px; cursor: pointer;",
                        onclick: close_modal,
                        "✕"
                    }
                }
                div {
                    style: "display: flex; flex-direction: column; gap: 12px;",
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center; padding: 8px 0;",
                        span { "Session ID" }
                        span { style: "color: #8696a0;", "{session.session_id}" }
                    }
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center; padding: 8px 0;",
                        span { "Connected User" }
                        span { style: "color: #8696a0;", "{session.push_name.as_deref().unwrap_or(\"Not connected\")}" }
                    }
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center; padding: 8px 0;",
                        span { "Phone / JID" }
                        span { style: "color: #8696a0; font-size: 13px;", "{session.jid.as_deref().unwrap_or(\"None\")}" }
                    }
                }
                button {
                    style: "margin-top: 12px; background: #e0483d; border: none; padding: 10px; border-radius: 8px; color: #ffffff; font-weight: 600; cursor: pointer;",
                    onclick: logout,
                    "Log Out"
                }
            }
        }
    }
}
