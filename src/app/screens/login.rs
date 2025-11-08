use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Paragraph},
};

use crate::{
    app::{
        components::input::Input,
        screens::{ConstaintDirection, ConstrainExtend, Renderable},
    },
    config::Config,
};

#[derive(Default, PartialEq)]
enum Selected {
    #[default]
    Username,
    Token,
    Button,
}

impl Selected {
    fn next(&self) -> Self {
        match self {
            Selected::Username => Selected::Token,
            Selected::Token => Selected::Button,
            Selected::Button => Selected::Username,
        }
    }

    fn prev(&self) -> Self {
        match self {
            Selected::Username => Selected::Button,
            Selected::Token => Selected::Username,
            Selected::Button => Selected::Token,
        }
    }
}

pub struct Login {
    selected: Selected,
    username: Input,
    token: Input,
}

impl Default for Login {
    fn default() -> Self {
        Self {
            selected: Default::default(),
            username: Input::new().selected().title("Username").build(),
            token: Input::password().title("Token").build(),
        }
    }
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

impl Login {
    fn select(&mut self, selected: Selected) {
        self.selected = selected;

        self.username
            .set_selected(self.selected == Selected::Username);
        self.token.set_selected(self.selected == Selected::Token);
    }

    fn submit(&mut self) {
        if self.username.value().len() == 0 {
            self.username.set_error(Some("Please enter username"));
            return;
        } else {
            self.username.set_error(None);
        }

        if self.token.value().len() == 0 {
            self.token.set_error(Some("Please enter token"));
            return;
        } else {
            self.token.set_error(None);
        }

        //TODO

        let config = Config {
            username: self.username.value().to_string(),
            token: self.token.value().to_string(),
        };

        config.save().unwrap();
    }
}

impl Renderable for Login {
    fn render(&mut self, frame: &mut ratatui::Frame) {
        let title = Line::from("Please login").bold().blue().centered();

        let card = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(title);

        let centered = center(
            frame.area(),
            Constraint::Percentage(25).max(&frame.area(), 40, ConstaintDirection::Horizontal),
            Constraint::Percentage(23).max(&frame.area(), 11, ConstaintDirection::Vertical),
        );

        let inside = card.inner(centered);
        frame.render_widget(card, centered);

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Fill(1),
            ])
            .split(inside);

        frame.render_widget(self.username.render(), main_layout[0]);
        frame.render_widget(self.token.render(), main_layout[1]);
        frame.render_widget(
            Paragraph::new("Login")
                .block(Block::bordered())
                .style(if self.selected == Selected::Button {
                    Style::new().fg(Color::Yellow)
                } else {
                    Style::new()
                })
                .centered(),
            main_layout[2],
        );
    }

    fn on_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab | KeyCode::Down => {
                self.select(self.selected.next());
            }
            KeyCode::Up => {
                self.select(self.selected.prev());
            }
            KeyCode::Enter => self.submit(),
            _ => match self.selected {
                Selected::Username => {
                    self.username.handle_key(key);
                }
                Selected::Token => {
                    self.token.handle_key(key);
                }
                Selected::Button => { /* EMPTY */ }
            },
        }
    }
}
