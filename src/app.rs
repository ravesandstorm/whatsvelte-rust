use dioxus::prelude::*;
use crate::state::AppState;
use crate::ui::main_layout::MainLayout;
use crate::ui::pairing_screen::PairingScreen;
use crate::ui::loading_screen::LoadingScreen;

#[component]
pub fn App() -> Element {
    let app_state = use_context_provider(AppState::new);
    let session = app_state.session.read();

    rsx! {
        if session.logged_in {
            MainLayout {}
        } else if session.registered {
            LoadingScreen {}
        } else {
            PairingScreen {}
        }
    }
}
