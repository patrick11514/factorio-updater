#![allow(dead_code)]

use crossterm::event::{Event, KeyEvent};
use derive_builder::Builder;
use ratatui::{
    style::{Color, Style},
    widgets::{Block, Paragraph},
};
use tui_input::{Input as NativeInput, backend::crossterm::EventHandler};

#[derive(Default, Debug, Clone)]
pub enum InputType {
    #[default]
    Text,
    Password,
}

#[derive(Default, Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct Input {
    native_input: NativeInput,
    #[builder(setter(strip_option))]
    error: Option<String>,
    selected_style: Style,
    unselected_style: Style,
    input_type: InputType,
    #[builder(default, setter(strip_option))]
    title: Option<String>,
    #[builder(default, setter(custom))]
    selected: bool,
}

impl InputBuilder {
    pub fn password() -> Self {
        let mut builder = Self::default();
        builder.input_type(InputType::Password);
        builder
    }

    pub fn selected(&mut self) -> &mut Self {
        self.selected = Some(true);
        self
    }

    pub fn with_value<T: Into<String>>(&mut self, value: T) -> &mut Self {
        let val = value.into();
        self.native_input = Some(NativeInput::default().with_value(val));
        self
    }
}

impl Input {
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

    pub fn handle_key(&mut self, key: &KeyEvent) {
        self.native_input.handle_event(&Event::Key(*key));
    }

    pub fn value(&self) -> &str {
        self.native_input.value()
    }
}
