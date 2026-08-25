use std::sync::Arc;
use dioxus::prelude::*;
use crate::dto::{ChatDto, ContactDto};
use crate::state::AppState;
use crate::session::SessionManager;
use crate::ui::avatar::Avatar;
use crate::ui::jid_utils::resolve_display_name;

fn format_chat_time(ts: i64) -> String {
    if ts <= 0 {
        return String::new();
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let diff = now - ts;
    // If message is > 1 day old (> 86400 seconds), show ONLY date DD/MM/YYYY
    if diff > 86400 {
        let days = ts / 86400;
        let z = days + 719468;
        let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
        let doe = (z - era * 146097) as u32;
        let yoe = (doe - doe / 1024 + doe / 1461 - doe / 142401) / 365;
        let y = yoe as i64 + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };
        format!("{:02}/{:02}/{:04}", d, m, y)
    } else {
        // Otherwise, show ONLY time HH:MM (24-hour)
        let secs_in_day = (ts % 86400 + 86400) % 86400;
        let hours = secs_in_day / 3600;
        let mins = (secs_in_day % 3600) / 60;
        format!("{:02}:{:02}", hours, mins)
    }
}

#[component]
pub fn ChatListItem(chat: ChatDto) -> Element {
    let mut app_state = use_context::<AppState>();
    let session_mgr = use_context::<Arc<SessionManager>>();
    let active_jid = app_state.active_chat_jid.read();
    let is_active = active_jid.as_deref() == Some(&chat.jid);
    let contacts = app_state.contacts.read();
    
    let background = if is_active { "#2a3942" } else { "transparent" };
    let chat_clone = chat.clone();
    let jid_clone = chat.jid.clone();
    let time_str = format_chat_time(chat.timestamp);

    let contact = contacts.get(&chat.jid);
    let contact_name = contact.and_then(|c| c.verified_name.as_deref().or(c.name.as_deref()));
    let display_name = resolve_display_name(&chat.jid, contact_name.or(chat.name.as_deref()), None);

    // Fetch profile picture if not cached
    let jid_for_pic = chat.jid.clone();
    let has_pic = contact.and_then(|c| c.picture_url.as_ref()).is_some();
    let is_logged_in = app_state.session.read().logged_in;
    use_effect(use_reactive(&has_pic, move |has_pic_val| {
        if !has_pic_val && is_logged_in {
            let jid = jid_for_pic.clone();
            let mgr = session_mgr.clone();
            let mut app_state = app_state;
            spawn(async move {
                if let Ok(Some(url)) = crate::commands::contacts::get_profile_picture_url(jid.clone(), Some(true), None, &mgr).await {
                    let mut contacts = app_state.contacts.write();
                    if let Some(c) = contacts.get_mut(&jid) {
                        c.picture_url = Some(url.clone());
                    } else {
                        contacts.insert(jid.clone(), ContactDto {
                            jid: jid.clone(),
                            name: None,
                            verified_name: None,
                            lid: None,
                            picture_url: Some(url.clone()),
                        });
                    }
                    if let Ok(c_json) = serde_json::to_string(&contacts.get(&jid)) {
                        let _ = document::eval(&format!("window.whatsappDb.saveContact({c_json});"));
                    }
                }
            });
        }
    }));

    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 15px; padding: 12px 16px; cursor: pointer; background-color: {background}; border-bottom: 1px solid #222e35; transition: background-color 0.15s ease;",
            onclick: move |_| {
                *app_state.active_chat_jid.write() = Some(jid_clone.clone());
            },
            // Avatar
            Avatar {
                jid: chat_clone.jid.clone(),
                name: Some(display_name.clone()),
                picture_url: contact.and_then(|c| c.picture_url.clone()),
                size: 48,
            }
            // Chat Details
            div {
                style: "flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 4px;",
                div {
                    style: "display: flex; justify-content: space-between; align-items: center;",
                    span {
                        style: "font-size: 16px; font-weight: 500; color: #e9edef; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                        "{display_name}"
                    }
                    if !time_str.is_empty() {
                        span {
                            style: "font-size: 12px; color: #8696a0; flex-shrink: 0; margin-left: 8px;",
                            "{time_str}"
                        }
                    }
                }
                div {
                    style: "display: flex; justify-content: space-between; align-items: center;",
                    p {
                        style: "font-size: 14px; color: #8696a0; margin: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                        "{chat_clone.last_message.as_deref().unwrap_or(\"\")}"
                    }
                    if chat_clone.unread > 0 {
                        span {
                            style: "background-color: #00a884; color: #111b21; font-size: 12px; font-weight: bold; border-radius: 10px; padding: 2px 6px; min-width: 18px; text-align: center; flex-shrink: 0; margin-left: 6px;",
                            "{chat_clone.unread}"
                        }
                    }
                }
            }
        }
    }
}
