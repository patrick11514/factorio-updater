use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::{Color, Style},
    text::Line,
    widgets::{Paragraph, Wrap},
};

use crate::app::{
    api::structs::ALL_PLATFORMS,
    components::popup::{PopupBuilder, PopupContent, PopupControl, PopupSize},
    screens::{
        ScreenEvent,
        main::{components::tick::OpenedPopup, screen::Main},
    },
};

pub async fn on_key(main: &mut Main, ev: &KeyEvent) -> Option<ScreenEvent> {
    match ev.code {
        KeyCode::Char('a') => {
            main.opened_popup = Some(OpenedPopup::VersionCreate(Default::default()));

            let lines = ALL_PLATFORMS
                .iter()
                .map(|arch| Line::from(arch.to_string()))
                .collect::<Vec<_>>();

            let paragraph = Paragraph::new("Select architecture:")
                .style(Style::default().fg(Color::Indexed(202)).bold())
                .centered()
                .wrap(Wrap { trim: false });

            let content: PopupContent = (lines, paragraph).into();

            Some(ScreenEvent::OpenPopup(
                PopupBuilder::default()
                    .title(Line::from("Version installation").centered())
                    .content(content)
                    .size(PopupSize::Medium)
                    .build()
                    .unwrap(),
            ))
        }
        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
            if let Some(popup) = &main.opened_popup {
                match popup {
                    OpenedPopup::VersionCreate(_) => {
                        Some(ScreenEvent::PopupControl(PopupControl::Previous))
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
            if let Some(popup) = &main.opened_popup {
                match popup {
                    OpenedPopup::VersionCreate(_) => {
                        Some(ScreenEvent::PopupControl(PopupControl::Next))
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    }
}
