//! Glue between the store/API and the Slint UI.

pub mod controller;
pub mod frame;
pub mod images;
pub mod qr;

use std::sync::Arc;

use slint::ComponentHandle;

use crate::{AppState, AppWindow};
use controller::Controller;

/// Wire every AppState callback to the controller. Async actions are spawned
/// on the tokio runtime so the UI thread never waits on the network.
pub fn install_callbacks(w: &AppWindow, ctrl: &Arc<Controller>) {
    let app = w.global::<AppState>();

    // Spawn `$body` (an async expression over `c`) on the runtime.
    macro_rules! spawn {
        ($c:ident, $body:expr) => {{
            let rt = $c.rt.clone();
            rt.spawn(async move { $body.await });
        }};
    }

    let c = ctrl.clone();
    app.on_select_chat(move |jid| {
        let c = c.clone();
        spawn!(c, c.select_chat(jid.to_string()));
    });
    let c = ctrl.clone();
    app.on_section_changed(move |s| c.set_section(s));
    let c = ctrl.clone();
    app.on_search_changed(move |q| c.set_search(q.to_string()));
    let c = ctrl.clone();
    app.on_new_chat(move |n| {
        let c = c.clone();
        spawn!(c, c.new_chat(n.to_string()));
    });

    let c = ctrl.clone();
    app.on_toggle_mute(move || {
        let c = c.clone();
        spawn!(c, c.toggle_flag("muted"));
    });
    let c = ctrl.clone();
    app.on_toggle_pin(move || {
        let c = c.clone();
        spawn!(c, c.toggle_flag("pinned"));
    });
    let c = ctrl.clone();
    app.on_toggle_archive(move || {
        let c = c.clone();
        spawn!(c, c.toggle_flag("archived"));
    });
    let c = ctrl.clone();
    app.on_load_older(move || c.load_older());

    let c = ctrl.clone();
    app.on_send(move |t| {
        let c = c.clone();
        spawn!(c, c.send(t.to_string()));
    });
    let c = ctrl.clone();
    app.on_composer_edited(move |t| c.composer_edited(t.to_string()));
    let c = ctrl.clone();
    app.on_cancel_compose(move || c.cancel_compose());
    let c = ctrl.clone();
    app.on_attach(move || {
        let c = c.clone();
        spawn!(c, c.attach());
    });

    let c = ctrl.clone();
    app.on_reply_to(move |id| c.reply_to(id.to_string()));
    let c = ctrl.clone();
    app.on_edit(move |id| c.start_edit(id.to_string()));
    let c = ctrl.clone();
    app.on_react(move |id, e| {
        let c = c.clone();
        spawn!(c, c.react(id.to_string(), e.to_string()));
    });
    let c = ctrl.clone();
    app.on_delete_for_me(move |id| {
        let c = c.clone();
        spawn!(c, c.delete_for_me(id.to_string()));
    });
    let c = ctrl.clone();
    app.on_delete_for_everyone(move |id| {
        let c = c.clone();
        spawn!(c, c.delete_for_everyone(id.to_string()));
    });
    let c = ctrl.clone();
    app.on_open_media(move |id| {
        let c = c.clone();
        spawn!(c, c.open_media(id.to_string()));
    });
    let c = ctrl.clone();
    app.on_save_media(move |id| {
        let c = c.clone();
        spawn!(c, c.save_media(id.to_string()));
    });
    let c = ctrl.clone();
    app.on_view_image(move |id| {
        let c = c.clone();
        spawn!(c, c.view_image(id.to_string()));
    });
    let c = ctrl.clone();
    app.on_retry(move |id| {
        let c = c.clone();
        spawn!(c, c.retry(id.to_string()));
    });

    let c = ctrl.clone();
    app.on_request_pair_code(move |p| {
        let c = c.clone();
        spawn!(c, c.request_pair_code(p.to_string()));
    });
    let c = ctrl.clone();
    app.on_reset_session(move || {
        let c = c.clone();
        spawn!(c, c.reset_session());
    });
    let c = ctrl.clone();
    app.on_logout(move || {
        let c = c.clone();
        spawn!(c, c.logout());
    });

    let c = ctrl.clone();
    app.on_theme_changed(move |t| c.update_settings(|s| s.theme = t.to_string()));
    let c = ctrl.clone();
    app.on_zoom_changed(move |z| c.update_settings(|s| s.zoom = z.clamp(0.8, 1.5)));
    let c = ctrl.clone();
    app.on_read_receipts_changed(move |v| c.update_settings(|s| s.send_read_receipts = v));
    let c = ctrl.clone();
    app.on_auto_load_changed(move |v| c.update_settings(|s| s.auto_load_images = v));
}
