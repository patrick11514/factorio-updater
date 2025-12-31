use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::Style,
    text::Line,
    widgets::{ListState, Paragraph, ScrollbarState, Wrap},
};

use crate::app::{
    api::structs::ALL_PLATFORMS,
    components::popup::{PopupBuilder, PopupContent, PopupControl, PopupSize, PopupType},
    screens::{
        ScreenEvent,
        main::{
            components::{
                popup_templates::install_popup, run::InstalledVersionState, tick::OpenedPopup,
            },
            screen::{Main, SelectedList},
        },
    },
    utils::ORANGE,
};

fn next(
    list: &mut ListState,
    scrollbar: &mut ScrollbarState,
    storage: &mut Option<usize>,
    len: usize,
    bar_rev: bool,
) {
    let current = list.selected();
    if let Some(idx) = current {
        if idx + 1 >= len {
            return;
        }
    }

    list.select_next();
    if bar_rev {
        scrollbar.prev();
    } else {
        scrollbar.next();
    }

    if let Some(idx) = list.selected() {
        *storage = Some(idx);
    } else {
        *storage = None;
    }
}

fn prev(
    list: &mut ListState,
    scrollbar: &mut ScrollbarState,
    storage: &mut Option<usize>,
    bar_rev: bool,
) {
    list.select_previous();
    if bar_rev {
        scrollbar.next();
    } else {
        scrollbar.prev();
    }

    if let Some(idx) = list.selected() {
        *storage = Some(idx);
    } else {
        *storage = None;
    }
}

fn move_list(
    list: &mut ListState,
    scrollbar: &mut ScrollbarState,
    len: usize,
    step: isize,
    bar_reversed: bool,
) -> Option<usize> {
    if len == 0 {
        list.select(None);
        return None;
    }

    let current = list.selected().map(|i| i as isize).unwrap_or(0);
    let new_idx = (current + step).clamp(0, (len as isize) - 1) as usize;

    list.select(Some(new_idx));

    match (step > 0, bar_reversed) {
        (true, false) | (false, true) => scrollbar.next(),
        _ => scrollbar.prev(),
    }

    Some(new_idx)
}

pub async fn on_key(main: &mut Main, ev: &KeyEvent) -> Option<ScreenEvent> {
    let res = match ev.code {
        KeyCode::Char('a') | KeyCode::Char('A') if main.opened_popup.is_none() => {
            main.opened_popup = Some(OpenedPopup::VersionCreate(Default::default()));

            let lines = ALL_PLATFORMS
                .iter()
                .map(|arch| Line::from(arch.to_string()))
                .collect::<Vec<_>>();

            let paragraph = Paragraph::new("Select architecture:")
                .style(Style::default().fg(ORANGE).bold())
                .centered()
                .wrap(Wrap { trim: false });

            let content: PopupContent = (lines, paragraph).into();

            let mut builder = PopupBuilder::default();
            install_popup(&mut builder);

            Some(ScreenEvent::OpenPopup(
                builder.content(content).build().unwrap(),
            ))
        }
        KeyCode::Char('u') | KeyCode::Char('U') if main.opened_popup.is_none() => {
            if let Some(idx) = main.selected_version {
                let details = main.installed_version_details.get(&idx);
                if let Some(details) = details {
                    if let InstalledVersionState::UpdateAvailable(update_type) = &details.state {
                        let popup = PopupBuilder::default()
                            .title("Update Version")
                            .size(PopupSize::Medium)
                            .content(
                                Paragraph::new(vec![
                                    Line::from("You are about to perform update:")
                                        .style(Style::default().fg(ORANGE).bold()),
                                    Line::from(update_type.to_string()),
                                ])
                                .wrap(Wrap { trim: false }),
                            )
                            .popup_type(PopupType::YesNo)
                            .build()
                            .unwrap();

                        main.opened_popup = Some(OpenedPopup::VersionUpdate(idx));

                        return Some(ScreenEvent::OpenPopup(popup));
                    }
                }
            }
            None
        }

        KeyCode::Char('q') | KeyCode::Esc => {
            main.opened_popup = None;
            //Real popup closed in App::on_key
            None
        }
        KeyCode::Char('l') | KeyCode::Char('L') if main.opened_popup.is_none() => {
            main.selected_list = SelectedList::Logs;
            main.version_list_state.select(None);
            main.logs_list_state.select(main.selected_log);
            None
        }
        KeyCode::Char('i') | KeyCode::Char('I') if main.opened_popup.is_none() => {
            main.selected_list = SelectedList::Versions;
            main.logs_list_state.select(None);

            let versions = Main::get_installed_versions(&main.api.config.installed_versions);

            main.version_list_state
                .select(versions.iter().position(|(uuid, _)| {
                    if let Some(selected) = &main.selected_version {
                        selected.to_string() == uuid.to_string()
                    } else {
                        false
                    }
                }));
            None
        }
        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
            if let Some(popup) = &main.opened_popup {
                match popup {
                    OpenedPopup::VersionCreate(_) => {
                        return Some(ScreenEvent::PopupControl(PopupControl::Previous));
                    }
                    _ => {}
                }
            }

            None
        }
        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
            if let Some(popup) = &main.opened_popup {
                match popup {
                    OpenedPopup::VersionCreate(_) => {
                        return Some(ScreenEvent::PopupControl(PopupControl::Next));
                    }
                    _ => {}
                }
            }

            None
        }
        _ => None,
    };

    if let Some(event) = res {
        return Some(event);
    }

    if main.opened_popup.is_some() {
        return None;
    }

    let step = match ev.code {
        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => -1,
        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => 1,
        _ => 0,
    };

    if step != 0 {
        match main.selected_list {
            SelectedList::Versions => {
                let version = Main::get_installed_versions(&main.api.config.installed_versions);

                let new_idx = move_list(
                    &mut main.version_list_state,
                    &mut main.version_scrollbar_state,
                    version.len(),
                    step,
                    false,
                );

                main.selected_version = new_idx.map(|i| version[i].0.clone());
            }
            SelectedList::Logs => {
                let new_idx = move_list(
                    &mut main.logs_list_state,
                    &mut main.logs_scrollbar_state,
                    main.logs.len(),
                    step * -1,
                    true,
                );

                main.selected_log = new_idx;
            }
        }
    }

    None
}
