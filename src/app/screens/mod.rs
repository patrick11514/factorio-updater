#![allow(dead_code)]

pub(crate) mod login;
pub(crate) mod main;

use async_trait::async_trait;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
};
use tokio::task::JoinHandle;

use crate::app::{
    api::Api,
    components::popup::{Popup, PopupResult},
};

#[async_trait]
pub trait Screen: Send {
    /// Method called once for initialization
    fn init(&mut self) -> Option<JoinHandle<()>> {
        None
    }
    /// Method called on every tick
    fn tick(&mut self) -> Option<ScreenEvent> {
        None
    }
    /// Method called on key event
    async fn on_key(&mut self, _: &KeyEvent) -> Option<ScreenEvent> {
        None
    }
    /// Method called on popup result
    async fn on_popup(&mut self, _: PopupResult) -> Option<ScreenEvent> {
        None
    }
    /// Method to render the screen
    fn render(&mut self, frame: &mut Frame);
}

pub enum ScreenEvent {
    Logged(Api),
    Logout,
    OpenPopup(Popup<'static>),
    ClosePopup,
    RunInit,
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
