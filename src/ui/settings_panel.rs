use dioxus::prelude::*;
use crate::state::AppState;

#[component]
pub fn SettingsPanel() -> Element {
    let mut app_state = use_context::<AppState>();
    let session = app_state.session.read();

    let close_modal = move |_| {
        *app_state.active_modal.write() = None;
    };

    rsx! {
        div {
            style: "position: fixed; inset: 0; background: rgba(0,0,0,0.6); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            div {
                style: "width: 450px; background: #202c33; border-radius: 12px; padding: 24px; color: #e9edef; display: flex; flex-direction: column; gap: 16px; box-shadow: 0 8px 32px rgba(0,0,0,0.4);",
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
                }
                button {
                    style: "margin-top: 12px; background: #e0483d; border: none; padding: 10px; border-radius: 8px; color: #ffffff; font-weight: 600; cursor: pointer;",
                    onclick: move |_| {
                        *app_state.session.write() = crate::dto::StatusDto {
                            session_id: "default".to_string(),
                            logged_in: false,
                            connected: false,
                            jid: None,
                            registered: false,
                            push_name: None,
                        };
                        *app_state.active_modal.write() = None;
                    },
                    "Log Out"
                }
            }
        }
    }
}
