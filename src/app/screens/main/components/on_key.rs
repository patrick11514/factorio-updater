use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::{Color, Style},
    text::Line,
    widgets::{ListState, Paragraph, ScrollbarState, Wrap},
};

use crate::app::{
    api::structs::ALL_PLATFORMS,
    components::popup::{PopupBuilder, PopupContent, PopupControl},
    screens::{
        ScreenEvent,
        main::{
            components::{popup_templates::install_popup, tick::OpenedPopup},
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

pub async fn on_key(main: &mut Main, ev: &KeyEvent) -> Option<ScreenEvent> {
    match ev.code {
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
        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
            if let Some(popup) = &main.opened_popup {
                match popup {
                    OpenedPopup::VersionCreate(_) => {
                        return Some(ScreenEvent::PopupControl(PopupControl::Previous));
                    }
                    _ => {}
                }
            } else if main.opened_popup.is_none() {
                match main.selected_list {
                    SelectedList::Versions => prev(
                        &mut main.version_list_state,
                        &mut main.version_scrollbar_state,
                        &mut main.selected_version,
                        false,
                    ),
                    SelectedList::Logs => next(
                        &mut main.logs_list_state,
                        &mut main.logs_scrollbar_state,
                        &mut main.selected_log,
                        main.logs.len(),
                        true,
                    ),
                };
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
            } else if main.opened_popup.is_none() {
                match main.selected_list {
                    SelectedList::Versions => next(
                        &mut main.version_list_state,
                        &mut main.version_scrollbar_state,
                        &mut main.selected_version,
                        main.api.config.installed_versions.len(),
                        false,
                    ),
                    SelectedList::Logs => prev(
                        &mut main.logs_list_state,
                        &mut main.logs_scrollbar_state,
                        &mut main.selected_log,
                        true,
                    ),
                };
            }
            None
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            main.opened_popup = None;
            //Real popup closed in App::on_key
            None
        }
        KeyCode::Char('l') | KeyCode::Char('L') => {
            main.selected_list = SelectedList::Logs;
            main.version_list_state.select(None);
            main.logs_list_state.select(main.selected_log);
            None
        }
        KeyCode::Char('i') | KeyCode::Char('I') => {
            main.selected_list = SelectedList::Versions;
            main.logs_list_state.select(None);
            main.version_list_state.select(main.selected_version);
            None
        }
        _ => None,
    }
}
