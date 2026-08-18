use dioxus::prelude::*;
use crate::state::AppState;
use crate::ui::connection_status::ConnectionStatus;
use crate::ui::qr_canvas::QrCanvas;

#[component]
pub fn PairingScreen() -> Element {
    let app_state = use_context::<AppState>();
    let qr_code = app_state.qr_code.read();

    rsx! {
        div {
            style: "height: 100vh; width: 100vw; display: grid; place-items: center; background-color: #111b21; padding: 24px; box-sizing: border-box;",
            div {
                style: "width: 760px; max-width: 95vw; background: #202c33; border-radius: 14px; padding: 36px 44px; display: grid; grid-template-columns: 1fr auto; gap: 48px; align-items: center; box-shadow: 0 10px 40px rgba(0, 0, 0, 0.4);",
                section {
                    style: "min-width: 0;",
                    header {
                        style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px;",
                        h1 { style: "font-size: 18px; margin: 0; color: #00a884;", "WhatsApp Dioxus Desktop" }
                        ConnectionStatus {}
                    }
                    h2 { style: "font-weight: 500; font-size: 22px; margin: 0 0 18px; color: #e9edef;", "Steps to log in" }
                    ol {
                        style: "color: #e9edef; font-size: 14px; line-height: 1.5; margin: 0 0 20px; padding-left: 20px; display: flex; flex-direction: column; gap: 12px;",
                        li { "Open ", b { style: "color: #00a884;", "WhatsApp" }, " on your phone" }
                        li { "Tap ", b { style: "color: #00a884;", "Menu" }, " on Android, or ", b { style: "color: #00a884;", "Settings" }, " on iPhone" }
                        li { "Tap ", b { style: "color: #00a884;", "Linked devices" }, ", then ", b { style: "color: #00a884;", "Link a device" } }
                        li { "Point your phone at this screen to scan the QR code" }
                    }
                }
                section {
                    style: "display: grid; place-items: center;",
                    if let Some(code) = qr_code.as_ref() {
                        QrCanvas { code: code.clone() }
                    } else {
                        div {
                            style: "width: 264px; height: 264px; border-radius: 12px; background: #111b21; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; color: #8696a0;",
                            div { style: "width: 34px; height: 34px; border: 3px solid #222e35; border-top-color: #00a884; border-radius: 50%; animation: spin 0.9s linear infinite;" }
                            span { "Waiting for QR..." }
                        }
                    }
                }
            }
        }
    }
}
