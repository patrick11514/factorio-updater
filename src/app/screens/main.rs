use async_trait::async_trait;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    symbols::border::Set,
    text::Line,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::app::{
    api::Api,
    screens::{Screen, ScreenEvent},
};

pub struct Main {
    username: String,
    api: Api,
}

impl Main {
    pub fn new(api: Api) -> Self {
        Self {
            username: api.config.username.clone(),
            api,
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

    async fn on_key(&mut self, _: KeyEvent) -> Option<ScreenEvent> {
        /*EMPTY */
        None
    }
}
