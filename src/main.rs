// Prevent a second console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    whatdioxus_rust_lib::run_dioxus()
}
