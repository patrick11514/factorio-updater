use async_trait::async_trait;
use crossterm::event::KeyEvent;
use ratatui::{
    style::Stylize,
    text::Line,
    widgets::{Block, BorderType, Paragraph},
};

use crate::app::{
    api::Api,
    components::popup::PopupResult,
    screens::{Screen, ScreenEvent},
};

pub struct Main {
    username: String,
    api: Api,
    count: usize,
}

impl Main {
    pub fn new(api: Api) -> Self {
        Self {
            username: api.config.username.clone(),
            api,
            count: 0,
        }
    }
}

#[async_trait]
impl Screen for Main {
    fn render(&mut self, frame: &mut ratatui::Frame) {
        let text = format!("Welcome {}", self.username);
        frame.render_widget(
            Paragraph::new(text)
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .title(Line::from(" Factorio Updater ").bold().blue().centered()),
                )
                .centered(),
            frame.area(),
        );
    }

    async fn on_key(&mut self, _: &KeyEvent) -> Option<ScreenEvent> {
        None
    }

    async fn on_popup(&mut self, _: PopupResult) -> Option<ScreenEvent> {
        None
    }
}
