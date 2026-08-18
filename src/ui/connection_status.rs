use dioxus::prelude::*;
use crate::state::AppState;

#[component]
pub fn ConnectionStatus() -> Element {
    let app_state = use_context::<AppState>();
    let session = app_state.session.read();

    let (status_text, badge_style) = if session.connected {
        ("Connected", "background: rgba(0, 168, 132, 0.2); color: #00a884; border: 1px solid rgba(0, 168, 132, 0.4);")
    } else if session.registered {
        ("Connecting...", "background: rgba(230, 162, 60, 0.2); color: #e6a23c; border: 1px solid rgba(230, 162, 60, 0.4);")
    } else {
        ("Disconnected", "background: rgba(245, 108, 108, 0.2); color: #f56c6c; border: 1px solid rgba(245, 108, 108, 0.4);")
    };

    rsx! {
        span {
            style: "padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: 500; {badge_style}",
            "{status_text}"
        }
    }
}
