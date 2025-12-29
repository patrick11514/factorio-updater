use std::sync::Arc;

use ratatui::text::Line;

use crate::app::{
    api::Response,
    components::popup::PopupBuilder,
    screens::{
        ScreenEvent,
        main::{components::run::RunState, message::MainMessage, screen::Main},
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
            MainMessage::LoadVersions(result, state) => match result {
                Ok(response) => match response {
                    Response::Success(updates) => {
                        state.lock().unwrap().finish();
                        main.updates = Some(Arc::new(updates));
                        main.run_state = RunState::CheckForUpdates;
                        return Some(ScreenEvent::RunInit);
                    }
                    Response::Error(error_response) => {
                        state.lock().unwrap().error();
                        main.opened_popup = Some(OpenedPopup::ErrorNotify);
                        return Some(ScreenEvent::OpenPopup(
                            PopupBuilder::error()
                                .title(Line::from("Fetching Versions").centered())
                                .content(format!(
                                    "Failed to fetch available versions:\n{}",
                                    error_response.message
                                ))
                                .build()
                                .unwrap(),
                        ));
                    }
                },
                Err(err) => {
                    state.lock().unwrap().error();
                    main.opened_popup = Some(OpenedPopup::ErrorNotify);
                    return Some(ScreenEvent::OpenPopup(
                        PopupBuilder::error()
                            .title(Line::from("Fetching Versions").centered())
                            .content(format!(
                                "An error occurred while fetching available versions:\n{}",
                                err
                            ))
                            .build()
                            .unwrap(),
                    ));
                }
            },
            MainMessage::ChangeRunState(run_state) => {
                main.run_state = run_state;
            }
            MainMessage::VersionDetails(items, state) => {
                main.installed_version_details = items;
                state.lock().unwrap().finish();
            }
        }
    }

    None
}
