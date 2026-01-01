use std::{env, path::Path};

use ratatui::{
    style::{Color, Style},
    text::Line,
    widgets::{Paragraph, Wrap},
};

use crate::app::{
    api::structs::{
        ALL_PLATFORMS, ALL_VERSIONS, Item, Platform, Version, get_versions_by_platform,
    },
    components::{
        input::InputBuilder,
        popup::{PopupBuilder, PopupContent, PopupControl, PopupSize, PopupType},
    },
    screens::{
        ScreenEvent,
        main::{
            components::{
                popup_templates::install_popup,
                run::{InstalledVersionState, RunState},
                tick::{OpenedPopup, VersionCreateStep},
            },
            screen::Main,
        },
    },
    utils::{ORANGE, get_sorted_updates},
    workflows::{delete_version, install_full_version, install_update},
};

pub struct VersionManage {}

impl VersionManage {
    pub fn select(main: &mut Main, state: &VersionCreateStep, idx: usize) -> Option<ScreenEvent> {
        let mut popup = PopupBuilder::default();
        install_popup(&mut popup);

        let (next_step, popup) = match state {
            VersionCreateStep::SelectingPlatform => {
                let platform = &ALL_PLATFORMS[idx];

                let lines = get_versions_by_platform(platform)
                    .iter()
                    .map(|version| Line::from(version.to_string()))
                    .collect::<Vec<_>>();

                let paragraph = Paragraph::new("Select edition:")
                    .style(Style::default().fg(ORANGE).bold())
                    .centered()
                    .wrap(Wrap { trim: false });

                let content: PopupContent = (lines, paragraph).into();

                (
                    VersionCreateStep::SelectingVersion {
                        platform: platform.clone(),
                    },
                    popup.content(content),
                )
            }
            VersionCreateStep::SelectingVersion { platform } => {
                let version = &ALL_VERSIONS[idx];

                let updates = main.updates.as_ref().unwrap().clone();
                let updates = get_sorted_updates(&updates, version, platform);

                let lines = updates
                    .iter()
                    .map(|version| {
                        Line::from(match version {
                            Item::VersionDiff(version_diff) => version_diff.from.clone(),
                            Item::Stable(stable) => {
                                format!("{} (stable)", stable.stable)
                            }
                        })
                    })
                    .collect::<Vec<_>>();

                let paragraph = Paragraph::new("Select version:")
                    .style(Style::default().fg(ORANGE).bold())
                    .centered()
                    .wrap(Wrap { trim: false });

                let content: PopupContent = (lines, paragraph).into();

                (
                    VersionCreateStep::SelectingPatch {
                        platform: platform.clone(),
                        version: version.clone(),
                    },
                    popup.content(content),
                )
            }
            VersionCreateStep::SelectingPatch { platform, version } => {
                let path = env::current_dir().unwrap().display().to_string();

                let updates = main.updates.as_ref().unwrap().clone();
                let updates = get_sorted_updates(&updates, version, platform);

                (
                    VersionCreateStep::SelectingPath {
                        platform: platform.clone(),
                        version: version.clone(),
                        patch: updates[idx].clone(),
                    },
                    popup.content(PopupContent::Input(
                        InputBuilder::path()
                            .title(
                                Line::from("Installation Path")
                                    .centered()
                                    .style(Style::default().fg(ORANGE).bold()),
                            )
                            .with_value(path)
                            .selected()
                            .build()
                            .unwrap(),
                    )),
                )
            }
            _ => panic!("Invalid state for version creation popup"),
        };

        main.opened_popup = Some(OpenedPopup::VersionCreate(next_step));

        Some(ScreenEvent::OpenPopup(popup.build().unwrap()))
    }

    pub fn open_summary_popup(
        main: &mut Main,
        platform: &Platform,
        version: &Version,
        patch: &Item,
        install_path: String,
    ) -> Option<ScreenEvent> {
        main.opened_popup = Some(OpenedPopup::VersionCreate(VersionCreateStep::Summary {
            platform: platform.clone(),
            version: version.clone(),
            patch: patch.clone(),
            install_path: install_path.clone(),
        }));

        return Some(ScreenEvent::OpenPopup(
            PopupBuilder::default()
                .title(Line::from("Installation Summary").centered())
                .popup_type(PopupType::YesNo)
                .size(PopupSize::Large)
                .content(PopupContent::Paragraph(Paragraph::new(vec![
                    Line::from("Confirm if everything matches")
                        .style(Style::default().fg(Color::Yellow))
                        .centered(),
                    Line::from(""),
                    Line::from(format!("Platform: {}", platform)),
                    Line::from(format!("Version: {}", version)),
                    Line::from(format!("Patch: {}", patch)),
                    Line::from(format!("Install Path: {}", install_path)),
                ])))
                .build()
                .unwrap(),
        ));
    }

    pub fn input(main: &mut Main, state: &VersionCreateStep, value: String) -> Option<ScreenEvent> {
        if let VersionCreateStep::SelectingPath {
            platform,
            version,
            patch,
        } = state
        {
            if value.is_empty() {
                return Some(ScreenEvent::PopupControl(PopupControl::SetError(
                    "Path cannot be empty".to_string(),
                )));
            }

            if !value.starts_with('/') {
                return Some(ScreenEvent::PopupControl(PopupControl::SetError(
                    "Path must start with '/'".to_string(),
                )));
            }

            if value.contains('\0') {
                return Some(ScreenEvent::PopupControl(PopupControl::SetError(
                    "Path contains null character".to_string(),
                )));
            }

            let forbidden_chars = ['<', '>', '|', '&', '$', '`', '\n', '\r'];
            if value.chars().any(|c| forbidden_chars.contains(&c)) {
                return Some(ScreenEvent::PopupControl(PopupControl::SetError(
                    "Path contains forbidden characters".to_string(),
                )));
            }

            //check if dir is empty
            let path = Path::new(&value);
            if path.exists() {
                let len = path.iter().count();
                log::debug!("Path exists with {} components", len);

                if len > 0 {
                    main.opened_popup = Some(OpenedPopup::VersionCreate(
                        VersionCreateStep::FolderNotEmpty {
                            platform: platform.clone(),
                            version: version.clone(),
                            patch: patch.clone(),
                            install_path: value.clone(),
                        },
                    ));

                    return Some(ScreenEvent::OpenPopup(
                        PopupBuilder::text(
                            "The selected folder is not empty. Do you want to continue?",
                        )
                        .popup_type(PopupType::YesNo)
                        .title(Line::from("Folder Not Empty").centered())
                        .build()
                        .unwrap(),
                    ));
                }
            }

            return VersionManage::open_summary_popup(main, platform, version, patch, value);
        }

        None
    }

    pub async fn question(main: &mut Main, state: &VersionCreateStep) -> Option<ScreenEvent> {
        match state {
            VersionCreateStep::FolderNotEmpty {
                platform,
                version,
                patch,
                install_path,
            } => VersionManage::open_summary_popup(
                main,
                platform,
                version,
                patch,
                install_path.clone(),
            ),
            VersionCreateStep::Summary {
                platform,
                version,
                patch,
                install_path,
            } => {
                let tx = main.tx.clone();
                let api = main.api.clone();
                let install_path = install_path.clone();
                let platform = platform.clone();
                let version = version.clone();
                let patch = patch.clone();

                tokio::spawn(async move {
                    install_full_version(tx, api, install_path, platform, version, patch).await
                });

                main.opened_popup = None;
                Some(ScreenEvent::ClosePopup)
            }
            _ => None,
        }
    }

    pub async fn update(main: &mut Main, idx: uuid::Uuid) -> Option<ScreenEvent> {
        let tx = main.tx.clone();
        let api = main.api.clone();
        let data = api.config.installed_versions.get(&idx).unwrap().clone();
        let details = main.installed_version_details.get_mut(&idx).unwrap();

        if let InstalledVersionState::UpdateAvailable(update) = &details.state {
            let update = update.clone();
            tokio::spawn(async move { install_update(tx, api, idx, data, update).await });
        }

        details.state = InstalledVersionState::Updating;
        main.opened_popup = None;
        Some(ScreenEvent::ClosePopup)
    }

    pub async fn delete(main: &mut Main, idx: uuid::Uuid) -> Option<ScreenEvent> {
        let tx = main.tx.clone();
        let data = main
            .api
            .config
            .installed_versions
            .get(&idx)
            .unwrap()
            .clone();

        tokio::spawn(async move {
            delete_version(tx, idx, data).await;
        });

        main.opened_popup = None;
        Some(ScreenEvent::ClosePopup)
    }
}
