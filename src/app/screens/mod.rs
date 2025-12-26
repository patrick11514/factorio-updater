#![allow(dead_code)]

pub(crate) mod login;
pub(crate) mod main;

use async_trait::async_trait;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
};

use crate::app::{
    api::Api,
    components::popup::{Popup, PopupResult},
};

#[async_trait]
pub trait Screen {
    fn render(&mut self, frame: &mut Frame);
    async fn on_key(&mut self, key: &KeyEvent) -> Option<ScreenEvent>;
    async fn on_popup(&mut self, result: PopupResult) -> Option<ScreenEvent>;
}

pub enum ScreenEvent {
    Logged(Api),
    OpenPopup(Popup<'static>),
    ClosePopup,
}

pub enum ConstaintDirection {
    Horizontal,
    Vertical,
}

trait ConstrainExtend {
    fn min(self, layout: &Rect, min: u16, direction: ConstaintDirection) -> Self;
    fn max(self, layout: &Rect, max: u16, direction: ConstaintDirection) -> Self;
}

impl ConstrainExtend for Constraint {
    fn min(self, layout: &Rect, min: u16, direction: ConstaintDirection) -> Self {
        let perc = match self {
            Constraint::Percentage(perc) => perc,
            c => return c,
        };
        let min = std::cmp::min(
            match direction {
                ConstaintDirection::Vertical => layout.height,
                ConstaintDirection::Horizontal => layout.width,
            } * perc
                / 100,
            min,
        );

        Constraint::Length(min)
    }

    fn max(self, layout: &Rect, max: u16, direction: ConstaintDirection) -> Self {
        let perc = match self {
            Constraint::Percentage(perc) => perc,
            c => return c,
        };
        let max = std::cmp::max(
            match direction {
                ConstaintDirection::Vertical => layout.height,
                ConstaintDirection::Horizontal => layout.width,
            } * perc
                / 100,
            max,
        );

        Constraint::Length(max)
    }
}
