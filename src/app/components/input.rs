#![allow(dead_code)]

use crossterm::event::{Event, KeyEvent};
use derive_builder::Builder;
use ratatui::{
    style::{Color, Style},
    text::Line,
    widgets::{Block, Paragraph, Wrap},
};
use tui_input::{Input as NativeInput, backend::crossterm::EventHandler};

#[derive(Default, Debug, Clone)]
pub enum InputType {
    #[default]
    Text,
    Password,
    Path,
}

#[derive(Default, Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct Input<'a> {
    #[builder(default)]
    native_input: NativeInput,
    #[builder(setter(strip_option), default)]
    error: Option<String>,
    #[builder(default)]
    selected_style: Style,
    #[builder(default)]
    unselected_style: Style,
    #[builder(default)]
    input_type: InputType,
    #[builder(default, setter(strip_option))]
    title: Option<Line<'a>>,
    #[builder(default, setter(custom))]
    selected: bool,
}

impl InputBuilder<'_> {
    pub fn password() -> Self {
        let mut builder = Self::default();
        builder.input_type(InputType::Password);
        builder
    }

    pub fn path() -> Self {
        let mut builder = Self::default();
        builder.input_type(InputType::Path);
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

impl Input<'_> {
    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    pub fn set_error(&mut self, error: Option<&str>) {
        self.error = error.map(|err| err.to_string());
    }

    pub fn have_error(&self) -> bool {
        self.error.is_some()
    }

    pub fn render(&mut self) -> Paragraph<'_> {
        let mut block = Block::bordered();

        // 1. Render the main Title (Top Left)
        if let Some(title) = &self.title {
            block = block.title(title.clone());
        }

        // 2. Render the Error (Bottom Left + Red Border)
        if let Some(error) = &self.error {
            // UX Tip: Turn the whole border red so the user notices immediately
            block = block.border_style(Style::default().fg(Color::Red));

            // Add the error message to the BOTTOM
            block = block.title_bottom(error.clone())
        } else {
            // Normal styling if no error
            if self.selected {
                block = block.style(self.selected_style);
            } else {
                block = block.style(self.unselected_style);
            }
        }

        Paragraph::new(match self.input_type {
            InputType::Text => self.native_input.value().to_string(),
            InputType::Password => "*".repeat(self.native_input.value().len()),
            InputType::Path => self.native_input.value().to_string(),
        })
        .wrap(Wrap { trim: false })
        .block(block)
    }

    pub fn handle_key(&mut self, key: &KeyEvent) {
        self.native_input.handle_event(&Event::Key(*key));
    }

    pub fn value(&self) -> &str {
        self.native_input.value()
    }
}
