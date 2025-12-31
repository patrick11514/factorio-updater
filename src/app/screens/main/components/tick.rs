use std::sync::Arc;

use ratatui::text::Line;

use crate::app::{
    api::{
        Response,
        structs::{Item, Platform, Version},
    },
    components::popup::PopupBuilder,
    screens::{
        ScreenEvent,
        main::{components::run::RunState, message::MainMessage, screen::Main},
    },
};

#[derive(Debug, Default, Clone)]
pub enum VersionCreateStep {
    #[default]
    SelectingPlatform,
    SelectingVersion {
        platform: Platform,
    },
    SelectingPatch {
        platform: Platform,
        version: Version,
    },
    SelectingPath {
        platform: Platform,
        version: Version,
        patch: Item,
    },
    FolderNotEmpty {
        platform: Platform,
        version: Version,
        patch: Item,
        install_path: String,
    },
    Summary {
        platform: Platform,
        version: Version,
        patch: Item,
        install_path: String,
    },
}

#[derive(Debug, Clone)]
pub enum OpenedPopup {
    LogoutNotify,
    ErrorNotify,
    VersionCreate(VersionCreateStep),
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
                        PopupBuilder::text("Username or token is invalid, please login again.")
                            .error()
                            .title(Line::from("Logout").centered())
                            .build()
                            .unwrap(),
                    ));
                }
                Err(err) => {
                    state.lock().unwrap().error();
                    main.opened_popup = Some(OpenedPopup::ErrorNotify);
                    return Some(ScreenEvent::OpenPopup(
                        PopupBuilder::text(format!(
                            "An error occurred while checking credentials:\n{}",
                            err
                        ))
                        .error()
                        .title(Line::from("Checking Credentials").centered())
                        .build()
                        .unwrap(),
                    ));
                }
            },
            MainMessage::CreateLog(log) => {
                main.logs.push(log);

                let len = main.logs.len();

                if let None = main.selected_log {
                    main.selected_log = Some(0);
                }

                main.logs_scrollbar_state = main
                    .logs_scrollbar_state
                    .content_length(len)
                    .position(len - main.selected_log.unwrap());
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
                            PopupBuilder::text(format!(
                                "Failed to fetch available versions:\n{}",
                                error_response.message
                            ))
                            .error()
                            .title(Line::from("Fetching Versions").centered())
                            .build()
                            .unwrap(),
                        ));
                    }
                },
                Err(err) => {
                    state.lock().unwrap().error();
                    main.opened_popup = Some(OpenedPopup::ErrorNotify);
                    return Some(ScreenEvent::OpenPopup(
                        PopupBuilder::text(format!(
                            "An error occurred while fetching available versions:\n{}",
                            err
                        ))
                        .error()
                        .title(Line::from("Fetching Versions").centered())
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
            MainMessage::OpenPopup((opened_popup, popup)) => {
                main.opened_popup = Some(opened_popup);
                return Some(ScreenEvent::OpenPopup(popup));
            }
            MainMessage::VersionInstalled(installed_version) => {
                main.api.config.installed_versions.push(installed_version);
                let config = main.api.config.clone();
                tokio::spawn(async move {
                    let _ = config.save().await;
                });
                main.run_state = RunState::CheckForUpdates;

                return Some(ScreenEvent::RunInit);
            }
        }
    }

    None
}
