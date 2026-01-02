use crate::app::{
    components::popup::PopupResult,
    screens::{
        ScreenEvent,
        main::{
            components::{
                on_popup::{installation::VersionManage, ok_result::handle_ok_result},
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
                VersionManage::select(main, state, idx)
            }
            (PopupResult::OkInput(value), OpenedPopup::VersionCreate(state)) => {
                VersionManage::input(main, state, value)
            }
            (PopupResult::Yes, OpenedPopup::VersionCreate(state)) => {
                VersionManage::question(main, state).await
            }
            (
                PopupResult::No,
                OpenedPopup::VersionCreate(_)
                | OpenedPopup::VersionUpdate(_)
                | OpenedPopup::VersionDelete(_),
            ) => {
                main.opened_popup = None;
                Some(ScreenEvent::ClosePopup)
            }
            (PopupResult::Yes, OpenedPopup::VersionUpdate(idx)) => {
                VersionManage::update(main, *idx).await
            }
            (PopupResult::Yes, OpenedPopup::VersionDelete(idx)) => {
                VersionManage::delete(main, *idx).await
            }
            _ => None,
        }
    } else {
        None
    }
}
