//! Wha.rs.app — native Slint desktop client for the whatsapp-rust library.
//!
//! One process, no WebView: the Slint event loop owns the main thread, a small
//! tokio runtime drives the WhatsApp clients and the UI controller. See
//! `docs/slint-migration.md`.

// Prevent a second console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;
mod media;
mod store;
mod ui;
mod util;

use std::sync::Arc;

use slint::ComponentHandle;

use crate::core::api::Api;
use crate::core::paths;
use crate::core::session::SessionManager;
use crate::ui::controller::{Controller, CTRL};

slint::include_modules!();

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Two workers are plenty for one account and keep idle memory down.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("wa-worker")
        .enable_all()
        .build()?;
    let _guard = rt.enter();

    let (ui_tx, ui_rx) = async_channel::unbounded();
    let manager = Arc::new(SessionManager::new(ui_tx, paths::data_dir()));
    let api = Api::new(manager);

    let window = AppWindow::new()?;
    let ctrl = Controller::new(api, rt.handle().clone(), window.as_weak());
    let _ = CTRL.set(ctrl.clone());
    ui::install_callbacks(&window, &ctrl);
    install_window_hooks(&window, &ctrl);

    #[cfg(debug_assertions)]
    let demo = std::env::var("WA_DEMO").is_ok();
    #[cfg(not(debug_assertions))]
    let demo = false;
    if demo {
        #[cfg(debug_assertions)]
        {
            rt.spawn(ctrl.clone().run_demo());
            rt.spawn(ctrl.clone().run_render_only());
        }
    } else {
        rt.spawn(ctrl.clone().run(ui_rx));
    }

    #[cfg(debug_assertions)]
    debug_snapshot(&window);

    window.run()?;

    ctrl.shutdown();
    // Don't wait for in-flight network calls on exit.
    rt.shutdown_timeout(std::time::Duration::from_millis(500));
    Ok(())
}

/// Window-level input the Slint tree doesn't see: global zoom shortcuts
/// (Ctrl/Cmd + `+`/`=`, `-`/`_`, `0`) and files dropped onto the window.
fn install_window_hooks(window: &AppWindow, ctrl: &Arc<Controller>) {
    use slint::winit_030::winit::event::{ElementState, WindowEvent};
    use slint::winit_030::winit::keyboard::{Key, ModifiersState};
    use slint::winit_030::{EventResult, WinitWindowAccessor};

    let ctrl = ctrl.clone();
    let mut mods = ModifiersState::empty();
    window.window().on_winit_window_event(move |_, event| {
        match event {
            WindowEvent::ModifiersChanged(m) => mods = m.state(),
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                let cmd = if cfg!(target_os = "macos") { mods.super_key() } else { mods.control_key() };
                if cmd && let Key::Character(c) = &event.logical_key {
                    let dir = match c.as_str() {
                        "+" | "=" => Some(1),
                        "-" | "_" => Some(-1),
                        "0" => Some(0),
                        _ => None,
                    };
                    if let Some(d) = dir {
                        ctrl.zoom_step(d);
                        return EventResult::PreventDefault;
                    }
                }
            }
            WindowEvent::DroppedFile(path) => ctrl.dropped_file(path.clone()),
            _ => {}
        }
        EventResult::Propagate
    });
}

/// Debug aid: `WA_SNAPSHOT=out.png` saves a render of the window after
/// `WA_SNAPSHOT_DELAY` seconds (default 8) and quits — lets the UI be checked
/// headlessly, without screen-recording permissions.
#[cfg(debug_assertions)]
fn debug_snapshot(window: &AppWindow) {
    let Ok(path) = std::env::var("WA_SNAPSHOT") else { return };
    let delay = std::env::var("WA_SNAPSHOT_DELAY").ok().and_then(|s| s.parse().ok()).unwrap_or(8);
    let weak = window.as_weak();
    slint::Timer::single_shot(std::time::Duration::from_secs(delay), move || {
        let Some(w) = weak.upgrade() else { return };
        match w.window().take_snapshot() {
            Ok(buf) => {
                let img = image::RgbaImage::from_raw(buf.width(), buf.height(), buf.as_bytes().to_vec());
                if let Some(img) = img
                    && let Err(e) = img.save(&path)
                {
                    log::error!("snapshot save failed: {e}");
                }
            }
            Err(e) => log::error!("snapshot failed: {e}"),
        }
        let _ = slint::quit_event_loop();
    });
}
