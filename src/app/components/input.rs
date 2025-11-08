#![allow(dead_code)]

use crossterm::event::{Event, KeyEvent};
use ratatui::{
    style::{Color, Style, Styled},
    widgets::{Block, Paragraph},
};
use tui_input::{Input as NativeInput, backend::crossterm::EventHandler};

#[derive(Default)]
pub enum InputType {
    #[default]
    Text,
    Password,
}

#[derive(Default)]
pub struct Input {
    native_input: NativeInput,
    error: Option<String>,
    selected_style: Style,
    unselected_style: Style,
    input_type: InputType,
    title: Option<String>,
    selected: bool,
}

impl Input {
    fn create_input(input_type: InputType) -> InputBuilder {
        InputBuilder {
            selected_style: Style::default().fg(ratatui::style::Color::Blue),
            unselected_style: Style::default(),
            input_type,
            title: None,
            selected: false,
        }
    }

    pub fn new() -> InputBuilder {
        Self::create_input(InputType::Text)
    }

    pub fn password() -> InputBuilder {
        Self::create_input(InputType::Password)
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    pub fn set_error(&mut self, error: Option<&str>) {
        self.error = match error {
            Some(err) => Some(err.to_string()),
            None => None,
        };
    }

    pub fn render(&mut self) -> Paragraph<'_> {
        let mut block = Block::bordered();

        if let Some(title) = &self.title {
            block = block.title(title.clone());
        }

        if let Some(error) = &self.error {
            block = block
                .title(error.clone())
                .title_style(Style::new().fg(Color::Red))
        }

        if self.selected {
            block = block.style(self.selected_style);
        } else {
            block = block.style(self.unselected_style);
        }

        Paragraph::new(match self.input_type {
            InputType::Text => self.native_input.value().to_string(),
            InputType::Password => "*".repeat(self.native_input.value().len()),
        })
        .block(block)
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        self.native_input.handle_event(&Event::Key(key));
    }

    pub fn value(&self) -> &str {
        self.native_input.value()
    }
}

pub struct InputBuilder {
    title: Option<String>,
    selected_style: Style,
    unselected_style: Style,
    input_type: InputType,
    selected: bool,
}

impl InputBuilder {
    pub fn title<T: Into<String>>(mut self, title: T) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.unselected_style = style;
        self
    }

    pub fn selected(mut self) -> Self {
        self.selected = true;
        self
    }

    pub fn build(self) -> Input {
        Input {
            native_input: NativeInput::default(),
            error: None,
            selected_style: self.selected_style,
            unselected_style: self.unselected_style,
            input_type: self.input_type,
            title: self.title,
            selected: self.selected,
        }
    }
}
