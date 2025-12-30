use std::env;

use ratatui::{
    style::{Color, Style},
    text::Line,
    widgets::{Paragraph, Wrap},
};

use crate::app::{
    api::structs::{ALL_PLATFORMS, ALL_VERSIONS, Arch, Item},
    components::popup::{PopupBuilder, PopupContent, PopupResult},
    screens::{
        Screen, ScreenEvent,
        main::{
            components::{
                popup_templates::install_popup,
                tick::{OpenedPopup, VersionCreateStep},
            },
            screen::Main,
        },
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
                _ => None,
            },
            PopupResult::OkSelect(idx) => match opened {
                OpenedPopup::VersionCreate(state) => {
                    let mut popup = PopupBuilder::default();
                    install_popup(&mut popup);

                    let (next_step, popup) = match state {
                        VersionCreateStep::SelectingPlatform => {
                            let lines = ALL_VERSIONS
                                .iter()
                                .map(|version| Line::from(version.to_string()))
                                .collect::<Vec<_>>();

                            let paragraph = Paragraph::new("Select edition:")
                                .style(Style::default().fg(Color::Indexed(202)).bold())
                                .centered()
                                .wrap(Wrap { trim: false });

                            let content: PopupContent = (lines, paragraph).into();

                            (
                                VersionCreateStep::SelectingVersion { platform: idx },
                                popup.content(content),
                            )
                        }
                        VersionCreateStep::SelectingVersion { platform } => {
                            let arch: Arch = (&ALL_VERSIONS[idx], &ALL_PLATFORMS[*platform]).into();
                            let updates = main.updates.as_ref().unwrap().clone();
                            let mut updates = updates.get(&arch).unwrap().clone();
                            updates.sort();
                            updates.reverse();

                            let lines = updates
                                .iter()
                                .map(|version| {
                                    Line::from(match version {
                                        Item::VersionDiff(version_diff) => {
                                            version_diff.from.clone()
                                        }
                                        Item::Stable(stable) => {
                                            format!("{} (stable)", stable.stable)
                                        }
                                    })
                                })
                                .collect::<Vec<_>>();

                            let paragraph = Paragraph::new("Select version:")
                                .style(Style::default().fg(Color::Indexed(202)).bold())
                                .centered()
                                .wrap(Wrap { trim: false });

                            let content: PopupContent = (lines, paragraph).into();

                            (
                                VersionCreateStep::SelectingPath {
                                    platform: *platform,
                                    version: idx,
                                },
                                popup.content(content),
                            )
                        }
                        _ => panic!("Invalid state for version creation popup"),
                    };

                    main.opened_popup = Some(OpenedPopup::VersionCreate(next_step));

                    Some(ScreenEvent::OpenPopup(popup.build().unwrap()))
                }
                _ => None,
            },
            PopupResult::OkInput(value) => match opened {
                OpenedPopup::VersionCreate(state) => {
                    let mut popup = PopupBuilder::default();
                    install_popup(&mut popup);

                    let (next_step, popup) = match state {
                        VersionCreateStep::SelectingPath {
                            platform: arch,
                            version,
                        } => {
                            let path = env::current_dir().unwrap().display().to_string();
                        
                            (VersionCreateStep::Selecting)
                        }
                        _ => panic!("Invalid state for version creation popup"),
                    }
                }
                _ => {}
            },
            _ => None,
        }
    } else {
        None
    }
}
