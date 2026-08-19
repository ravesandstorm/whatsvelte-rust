use dioxus::prelude::*;
use crate::state::AppState;
use crate::ui::chat_list::ChatList;
use crate::ui::conversation::Conversation;
use crate::ui::settings_panel::SettingsPanel;

#[component]
pub fn MainLayout() -> Element {
    let app_state = use_context::<AppState>();
    let active_modal = app_state.active_modal.read();

    rsx! {
        div {
            style: "width: 100%; height: 100%; display: flex; overflow: hidden; background-color: #111b21;",
            ChatList {}
            Conversation {}
            if active_modal.as_deref() == Some("settings") {
                SettingsPanel {}
            }
        }
    }
}
