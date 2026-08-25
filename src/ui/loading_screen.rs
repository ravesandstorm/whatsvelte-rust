use dioxus::prelude::*;
use crate::state::AppState;
use crate::ui::connection_status::ConnectionStatus;

#[component]
pub fn LoadingScreen() -> Element {
    let mut app_state = use_context::<AppState>();
    let progress = app_state.sync_progress.read();
    let chats_count = app_state.chats.read().len();

    let (title, detail, pct) = if let Some(p) = progress.as_ref() {
        let percent = if p.total > 0 {
            ((p.loaded as f32 / p.total as f32) * 100.0).min(100.0) as u32
        } else {
            0
        };
        let detail_str = if p.total > 0 {
            format!("Synced {} of {} messages ({}%)", p.loaded, p.total, percent)
        } else if p.loaded > 0 {
            format!("Synced {} messages", p.loaded)
        } else {
            "Connecting to WhatsApp servers...".to_string()
        };
        (p.title.clone(), detail_str, percent)
    } else {
        (
            "Loading your chats...".to_string(),
            "Retrieving conversation history from your device".to_string(),
            0,
        )
    };

    rsx! {
        div {
            style: "height: 100%; width: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 20px; background-color: #111b21; color: #e9edef; padding: 24px; box-sizing: border-box;",
            div {
                style: "width: 48px; height: 48px; border: 4px solid #222e35; border-top-color: #00a884; border-radius: 50%; animation: spin 0.9s linear infinite;",
            }
            div {
                style: "text-align: center; max-width: 400px; width: 100%;",
                h3 {
                    style: "font-size: 18px; font-weight: 500; color: #e9edef; margin: 0 0 8px 0;",
                    "{title}"
                }
                p {
                    style: "font-size: 13.5px; color: #8696a0; margin: 0 0 16px 0;",
                    "{detail}"
                }
                if progress.as_ref().map(|p| p.total > 0).unwrap_or(false) {
                    div {
                        style: "width: 100%; height: 6px; background-color: #202c33; border-radius: 3px; overflow: hidden; margin-bottom: 12px;",
                        div {
                            style: "height: 100%; width: {pct}%; background-color: #00a884; transition: width 0.3s ease;",
                        }
                    }
                }
            }
            div {
                style: "display: flex; align-items: center; gap: 12px;",
                ConnectionStatus {}
                if chats_count > 0 {
                    button {
                        style: "background: #00a884; border: none; border-radius: 8px; color: #111b21; font-weight: 600; font-size: 13px; padding: 6px 14px; cursor: pointer;",
                        onclick: move |_| {
                            app_state.session.write().logged_in = true;
                        },
                        "Open Chats ({chats_count})"
                    }
                }
            }
        }
    }
}
