use crossterm::event::KeyEvent;
use ratatui::{
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
};

use crate::{app::screens::Renderable, config::Config};

pub struct Main {
    config: Config,
}

impl Main {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

impl Renderable for Main {
    fn render(&mut self, frame: &mut ratatui::Frame) {
        let title = Line::from("Main").bold().blue().centered();
        let text = format!("Welcome {}", self.config.username);
        frame.render_widget(
            Paragraph::new(text)
                .block(Block::bordered().title(title))
                .centered(),
            frame.area(),
        )
    }

    fn on_key(&mut self, _: KeyEvent) { /*EMPTY */
    }
}
