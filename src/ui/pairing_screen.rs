use std::sync::Arc;
use dioxus::prelude::*;
use crate::state::AppState;
use crate::session::SessionManager;
use crate::ui::connection_status::ConnectionStatus;
use crate::ui::qr_canvas::QrCanvas;

#[component]
pub fn PairingScreen() -> Element {
    let app_state = use_context::<AppState>();
    let session_mgr = use_context::<Arc<SessionManager>>();
    let qr_code = app_state.qr_code.read();
    let pair_code = app_state.pair_code.read();

    let mut use_phone_mode = use_signal(|| false);
    let mut phone_input = use_signal(String::new);
    let mut loading_code = use_signal(|| false);
    let mut error_msg = use_signal(|| Option::<String>::None);

    let request_pair_code = {
        let mgr = session_mgr.clone();
        move |_| {
            let phone = phone_input.read().trim().to_string();
            if phone.is_empty() {
                return;
            }
            loading_code.set(true);
            error_msg.set(None);
            let mgr_clone = mgr.clone();
            let mut app_state = app_state;
            spawn(async move {
                match crate::commands::auth::auth_start_pair_code(phone, None, None, &mgr_clone).await {
                    Ok(dto) => {
                        *app_state.pair_code.write() = Some(dto.code);
                        loading_code.set(false);
                    }
                    Err(e) => {
                        error_msg.set(Some(format!("Failed to get code: {e}")));
                        loading_code.set(false);
                    }
                }
            });
        }
    };

    let refresh_qr = {
        let mgr = session_mgr.clone();
        let mut app_state = app_state;
        move |_| {
            let mgr_clone = mgr.clone();
            *app_state.qr_code.write() = None;
            spawn(async move {
                let _ = mgr_clone.reset("default").await;
            });
        }
    };


    rsx! {
        div {
            style: "height: 100%; width: 100%; display: grid; place-items: center; background-color: #111b21; padding: 24px; box-sizing: border-box; overflow-y: auto;",
            div {
                style: "width: 760px; max-width: 95vw; background: #202c33; border-radius: 14px; padding: 36px 44px; display: grid; grid-template-columns: 1fr auto; gap: 48px; align-items: center; box-shadow: 0 10px 40px rgba(0, 0, 0, 0.4);",
                section {
                    style: "min-width: 0;",
                    header {
                        style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px;",
                        h1 { style: "font-size: 18px; margin: 0; color: #00a884; font-weight: 600;", "WhatsApp Desktop" }
                        ConnectionStatus {}
                    }
                    h2 { style: "font-weight: 500; font-size: 22px; margin: 0 0 18px; color: #e9edef;", "Steps to log in" }
                    ol {
                        style: "color: #e9edef; font-size: 14px; line-height: 1.5; margin: 0 0 20px; padding-left: 20px; display: flex; flex-direction: column; gap: 12px;",
                        li { "Open ", b { style: "color: #00a884;", "WhatsApp" }, " on your phone" }
                        li { "Tap ", b { style: "color: #00a884;", "Menu" }, " on Android, or ", b { style: "color: #00a884;", "Settings" }, " on iPhone" }
                        li { "Tap ", b { style: "color: #00a884;", "Linked devices" }, ", then ", b { style: "color: #00a884;", "Link a device" } }
                        li {
                            if *use_phone_mode.read() {
                                "Tap 'Link with phone number instead' and enter the 8-digit code"
                            } else {
                                "Point your phone at this screen to scan the QR code"
                            }
                        }
                    }
                    div {
                        style: "margin-top: 16px;",
                        button {
                            style: "background: none; border: none; color: #00a884; font-size: 14px; cursor: pointer; padding: 0; text-decoration: underline;",
                            onclick: move |_| {
                                let cur = *use_phone_mode.read();
                                use_phone_mode.set(!cur);
                            },
                            if *use_phone_mode.read() {
                                "Link with QR code instead"
                            } else {
                                "Link with phone number instead"
                            }
                        }
                    }
                }
                section {
                    style: "display: grid; place-items: center;",
                    if *use_phone_mode.read() {
                        div {
                            style: "width: 264px; min-height: 264px; border-radius: 12px; background: #111b21; padding: 20px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 16px; box-sizing: border-box;",
                            if let Some(code) = pair_code.as_ref() {
                                div {
                                    style: "text-align: center;",
                                    span { style: "color: #8696a0; font-size: 12px; display: block; margin-bottom: 8px;", "Enter this code on your phone:" }
                                    div {
                                        style: "font-size: 24px; font-weight: 700; letter-spacing: 4px; color: #00a884; background: #202c33; padding: 10px 16px; border-radius: 8px;",
                                        "{code}"
                                    }
                                }
                            } else {
                                div {
                                    style: "width: 100%; display: flex; flex-direction: column; gap: 10px;",
                                    label { style: "font-size: 12px; color: #8696a0;", "Enter phone number with country code:" }
                                    input {
                                        style: "width: 100%; height: 36px; background: #202c33; border: 1px solid #374248; border-radius: 6px; padding: 0 10px; color: #e9edef; font-size: 14px; outline: none; box-sizing: border-box;",
                                        placeholder: "e.g. 15551234567",
                                        value: "{phone_input}",
                                        oninput: move |e| phone_input.set(e.value())
                                    }
                                    if let Some(err) = error_msg.read().as_ref() {
                                        span { style: "font-size: 11px; color: #f56c6c;", "{err}" }
                                    }
                                    button {
                                        style: "height: 36px; background: #00a884; border: none; border-radius: 6px; color: #111b21; font-weight: 600; cursor: pointer;",
                                        disabled: *loading_code.read(),
                                        onclick: request_pair_code,
                                        if *loading_code.read() { "Getting code..." } else { "Get Code" }
                                    }
                                }
                            }
                        }
                    } else if let Some(code) = qr_code.as_ref() {
                        QrCanvas { code: code.clone() }
                    } else {
                        div {
                            style: "width: 264px; height: 264px; border-radius: 12px; background: #111b21; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; color: #8696a0;",
                            div { style: "width: 34px; height: 34px; border: 3px solid #222e35; border-top-color: #00a884; border-radius: 50%; animation: spin 0.9s linear infinite;" }
                            span { "Loading QR code..." }
                            button {
                                style: "background: none; border: 1px solid #374248; border-radius: 6px; color: #8696a0; font-size: 12px; padding: 4px 10px; cursor: pointer; margin-top: 4px;",
                                onclick: refresh_qr,
                                "Reload"
                            }
                        }
                    }
                }
            }
        }
    }
}
