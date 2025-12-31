use crate::app::{
    components::popup::PopupResult,
    screens::{
        ScreenEvent,
        main::{
            components::{
                on_popup::{installation::VersionInstall, ok_result::handle_ok_result},
                tick::OpenedPopup,
            },
            screen::Main,
        },
    },
};

mod installation;
mod ok_result;

pub async fn on_popup(main: &mut Main, res: PopupResult) -> Option<ScreenEvent> {
    let popup = main.opened_popup.clone();

    if let Some(opened) = popup {
        match (res, &opened) {
            (PopupResult::Ok, _) => handle_ok_result(main, &opened),
            (PopupResult::OkSelect(idx), OpenedPopup::VersionCreate(state)) => {
                VersionInstall::select(main, state, idx)
            }
            (PopupResult::OkInput(value), OpenedPopup::VersionCreate(state)) => {
                VersionInstall::input(main, state, value)
            }
            (PopupResult::Yes, OpenedPopup::VersionCreate(state)) => {
                VersionInstall::question(main, state).await
            }
            (PopupResult::No, OpenedPopup::VersionCreate(_))
            | (PopupResult::No, OpenedPopup::VersionUpdate(_)) => {
                main.opened_popup = None;
                Some(ScreenEvent::ClosePopup)
            }
            /*(PopupResult::Yes, OpenedPopup::VersionUpdate(idx)) => {
                VersionInstall::update(main).await
            }*/
            _ => None,
        }
    } else {
        None
    }
}
