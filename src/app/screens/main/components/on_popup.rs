use crate::app::{
    components::popup::PopupResult,
    screens::{
        Screen, ScreenEvent,
        main::{components::tick::OpenedPopup, screen::Main},
    },
};

pub async fn on_popup(main: &mut Main, res: PopupResult) -> Option<ScreenEvent> {
    if let Some(opened) = &main.opened_popup {
        match res {
            PopupResult::Ok => match opened {
                OpenedPopup::LogoutNotify => Some(ScreenEvent::Logout),
                OpenedPopup::ErrorNotify => {
                    main.opened_popup = None;

                    // Retry checking credentials
                    main.init();

                    Some(ScreenEvent::ClosePopup)
                }
                OpenedPopup::VersionCreate(_) => {
                    main.opened_popup = None;
                    Some(ScreenEvent::ClosePopup)
                }
            },
            _ => None,
        }
    } else {
        None
    }
}
