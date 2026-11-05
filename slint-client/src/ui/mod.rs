//! Glue between the store/API and the Slint UI.

pub mod actions;
pub mod attach;
pub mod controller;
#[cfg(debug_assertions)]
pub mod demo;
pub mod frame;
pub mod images;
pub mod qr;
pub mod views;

use std::sync::Arc;

use slint::ComponentHandle;

use crate::{AppState, AppWindow};
use controller::Controller;

/// Wire every AppState callback to the controller. Async actions are spawned
/// on the tokio runtime so the UI thread never waits on the network.
pub fn install_callbacks(w: &AppWindow, ctrl: &Arc<Controller>) {
    let app = w.global::<AppState>();

    // `on!(name, |args| expr)` — synchronous handler.
    macro_rules! on {
        ($setter:ident, $c:ident => |$($a:ident),*| $body:expr) => {{
            let $c = ctrl.clone();
            app.$setter(move |$($a),*| $body);
        }};
    }
    // `spawn!(name, |args| async-expr)` — runs on the tokio runtime.
    macro_rules! spawn {
        ($setter:ident, $c:ident => |$($a:ident),*| $body:expr) => {{
            let c0 = ctrl.clone();
            app.$setter(move |$($a),*| {
                let $c = c0.clone();
                c0.rt.spawn(async move { $body.await });
            });
        }};
    }

    // pairing / loading
    spawn!(on_request_pair_code, c => |p| c.request_pair_code(p.to_string()));
    spawn!(on_reset_session, c => | | c.reset_session());
    spawn!(on_logout, c => | | c.logout());

    // chat list
    spawn!(on_select_chat, c => |jid| c.select_chat(jid.to_string()));
    on!(on_section_changed, c => |s| c.set_section(s));
    on!(on_search_changed, c => |q| c.set_search(q.to_string()));
    spawn!(on_new_chat, c => |n| c.new_chat(n.to_string()));
    spawn!(on_chat_action, c => |jid, flag| c.chat_action(jid.to_string(), flag.to_string()));
    on!(on_need_avatar, c => |jid| c.need_avatar(jid.to_string()));

    // header
    spawn!(on_toggle_mute, c => | | c.toggle_active_flag("muted"));
    spawn!(on_toggle_pin, c => | | c.toggle_active_flag("pinned"));
    spawn!(on_toggle_archive, c => | | c.toggle_active_flag("archived"));

    // message list
    on!(on_load_older, c => | | c.load_older());
    on!(on_seen, c => |id| c.seen(id.to_string()));
    on!(on_reached_bottom, c => | | c.reached_bottom());
    on!(on_jump_to_bottom, c => | | c.jump_to_bottom());
    on!(on_reveal_quote, c => |id| c.reveal_quote(id.to_string()));
    on!(on_open_link, c => |url| c.open_link(url.to_string()));

    // composer
    spawn!(on_send, c => |t| c.send(t.to_string()));
    on!(on_composer_edited, c => |t, caret| c.composer_edited(t.to_string(), caret.max(0) as usize));
    on!(on_cancel_compose, c => | | c.cancel_compose());
    on!(on_accept_suggestion, c => |i| c.accept_suggestion(i.max(0) as usize));
    on!(on_paste_image, c => | | c.paste_image());

    // attachments
    spawn!(on_attach_photos, c => | | c.attach_photos());
    spawn!(on_attach_document, c => | | c.attach_document());
    on!(on_attach_camera, c => | | c.attach_camera());
    on!(on_attach_voice, c => | | c.attach_voice());
    spawn!(on_preview_send, c => |caption, hd| c.preview_send(caption.to_string(), hd));
    on!(on_preview_cancel, c => | | c.preview_cancel());
    on!(on_preview_play, c => | | c.preview_play());
    spawn!(on_rec_stop, c => | | c.rec_stop());
    spawn!(on_rec_capture, c => | | c.rec_capture());
    on!(on_rec_cancel, c => | | c.rec_cancel());

    // emoji / gif panel
    on!(on_picker_tab_changed, c => |t| c.set_picker_tab(t));
    on!(on_emoji_tab_changed, c => |t| c.set_emoji_tab(t));
    on!(on_emoji_search_changed, c => |q| c.set_emoji_search(q.to_string()));
    on!(on_emoji_picked, c => |e, caret| c.emoji_picked(e.to_string(), caret.max(0) as usize));
    on!(on_gif_search, c => |q| c.gif_search(q.to_string()));
    on!(on_gif_more, c => | | c.gif_more());
    spawn!(on_gif_picked, c => |id| c.gif_picked(id.to_string()));

    // message actions
    on!(on_reply_to, c => |id| c.reply_to(id.to_string()));
    on!(on_edit, c => |id| c.start_edit(id.to_string()));
    on!(on_open_reactions, c => |id| c.open_reactions(id.to_string()));
    spawn!(on_react, c => |id, e| c.react(id.to_string(), e.to_string()));
    on!(on_ask_delete, c => |id| c.ask_delete(id.to_string()));
    spawn!(on_delete_for_me, c => |id| c.delete_for_me(id.to_string()));
    spawn!(on_delete_for_everyone, c => |id| c.delete_for_everyone(id.to_string()));
    spawn!(on_open_media, c => |id| c.open_media(id.to_string()));
    spawn!(on_save_media, c => |id| c.save_media(id.to_string()));
    on!(on_view_image, c => |id| c.view_image(id.to_string()));
    spawn!(on_toggle_audio, c => |id| c.toggle_audio(id.to_string()));
    on!(on_open_album, c => |id| c.open_album(id.to_string()));
    spawn!(on_retry, c => |id| c.retry(id.to_string()));

    // lightbox
    on!(on_viewer_step, c => |d| c.viewer_step(d));
    spawn!(on_viewer_play, c => | | c.viewer_play());
    on!(on_close_viewer, c => | | c.close_viewer());

    // toasts
    on!(on_dismiss_toast, c => |i| c.dismiss_toast(i.max(0) as usize));

    // settings
    on!(on_theme_changed, c => |t| c.update_settings(|s| s.theme = t.to_string()));
    on!(on_zoom_changed, c => |z| c.update_settings(|s| s.set_zoom(z)));
    on!(on_read_receipts_changed, c => |v| c.update_settings(|s| s.send_read_receipts = v));
    on!(on_auto_load_changed, c => |v| c.update_settings(|s| s.auto_load_images = v));
    on!(on_enter_to_send_changed, c => |v| c.update_settings(|s| s.enter_to_send = v));
    spawn!(on_wallpaper_changed, c => |w| c.set_wallpaper(w.to_string()));
}
