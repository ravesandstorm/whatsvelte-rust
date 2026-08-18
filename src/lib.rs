//! Pure Rust Dioxus WhatsApp Desktop Application.

pub mod app;
pub mod state;
pub mod ui;
mod bridge;
mod commands;
mod dto;
mod error;
mod session;

pub fn run_dioxus() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    dioxus::launch(app::App);
}
