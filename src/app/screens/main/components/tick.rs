use ratatui::text::Line;

use crate::app::{
    components::popup::PopupBuilder,
    screens::{
        ScreenEvent,
        main::{message::MainMessage, screen::Main},
    },
};

pub enum OpenedPopup {
    LogoutNotify,
    ErrorNotify,
}

pub fn tick(main: &mut Main) -> Option<ScreenEvent> {
    while let Ok(msg) = main.rx.try_recv() {
        match msg {
            MainMessage::CheckLogin(result, state) => match result {
                Ok(Some(())) => {
                    state.lock().unwrap().finish();
                }
                Ok(None) => {
                    state.lock().unwrap().error();
                    main.opened_popup = Some(OpenedPopup::LogoutNotify);
                    return Some(ScreenEvent::OpenPopup(
                        PopupBuilder::error()
                            .title(Line::from("Logout").centered())
                            .content("Username or token is invalid, please login again.")
                            .build()
                            .unwrap(),
                    ));
                }
                Err(err) => {
                    state.lock().unwrap().error();
                    main.opened_popup = Some(OpenedPopup::ErrorNotify);
                    return Some(ScreenEvent::OpenPopup(
                        PopupBuilder::error()
                            .title(Line::from("Checking Credentials").centered())
                            .content(format!(
                                "An error occurred while checking credentials:\n{}",
                                err
                            ))
                            .build()
                            .unwrap(),
                    ));
                }
            },
            MainMessage::CreateLog(log) => {
                main.logs.push(Box::new(log));
            }
        }
    }

    None
}
