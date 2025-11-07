use ratatui::{
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
};

use crate::app::screens::Renderable;

pub struct Login {}

impl Renderable for Login {
    fn render(&mut self, frame: &mut ratatui::Frame) {
        let title = Line::from("Login screen").bold().blue().centered();
        let text = "Please login :)";
        frame.render_widget(
            Paragraph::new(text)
                .block(Block::bordered().title(title))
                .centered(),
            frame.area(),
        )
    }
}
