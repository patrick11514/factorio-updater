use crate::app::screens::{
    Screen, ScreenEvent,
    main::{components::tick::OpenedPopup, screen::Main},
};

pub fn handle_ok_result(main: &mut Main, opened: &OpenedPopup) -> Option<ScreenEvent> {
    match opened {
        OpenedPopup::LogoutNotify => Some(ScreenEvent::Logout),
        OpenedPopup::ErrorNotify => {
            main.opened_popup = None;

            // Retry checking credentials
            main.init();

            Some(ScreenEvent::ClosePopup)
        }
        _ => None,
    }
}
