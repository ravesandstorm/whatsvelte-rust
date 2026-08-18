use dioxus::prelude::*;
use crate::ui::connection_status::ConnectionStatus;

#[component]
pub fn LoadingScreen() -> Element {
    rsx! {
        div {
            style: "height: 100vh; width: 100vw; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 16px; background-color: #111b21; color: #e9edef;",
            div {
                style: "width: 38px; height: 38px; border: 3px solid #222e35; border-top-color: #00a884; border-radius: 50%; animation: spin 0.9s linear infinite;",
            }
            p {
                style: "font-size: 15px; color: #e9edef; margin: 0;",
                "Loading your chats..."
            }
            ConnectionStatus {}
        }
    }
}
