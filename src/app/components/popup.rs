#![allow(dead_code)]

use crossterm::event::{KeyCode, KeyEvent};
use derive_builder::Builder;
use ratatui::{
    buffer::Buffer,
    layout::{self, Constraint, Layout, Rect},
    style::{self, Style},
    text::{Line, Text},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListState, Paragraph, Scrollbar, ScrollbarState,
        StatefulWidget, Widget, Wrap,
    },
};

use crate::app::{
    components::input::{Input, InputBuilder},
    utils::{style_list, style_scrollbar},
};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum PopupType {
    #[default]
    Ok,
    YesNo,
}

#[derive(Debug, Clone)]
pub enum PopupResult {
    Ok,
    OkSelect(usize),
    OkInput(String),
    Yes,
    No,
}

#[derive(Debug, Clone)]
pub enum PopupContent {
    Text(Line<'static>),
    Paragraph(Paragraph<'static>),
    Select(
        ListState,
        ScrollbarState,
        Vec<Line<'static>>,
        Option<Paragraph<'static>>,
    ),
    Input(Input<'static>),
}

impl Default for PopupContent {
    fn default() -> Self {
        PopupContent::Text(Line::from(""))
    }
}

impl From<Line<'static>> for PopupContent {
    fn from(line: Line<'static>) -> Self {
        PopupContent::Text(line)
    }
}

impl From<Paragraph<'static>> for PopupContent {
    fn from(paragraph: Paragraph<'static>) -> Self {
        PopupContent::Paragraph(paragraph)
    }
}

fn initialize_states(lines: &Vec<Line<'static>>) -> (ListState, ScrollbarState) {
    let mut state = ListState::default();
    let mut scrollbar_state = ScrollbarState::default().content_length(lines.len());

    if !lines.is_empty() {
        state.select(Some(0));
        scrollbar_state = scrollbar_state.position(0);
    }

    (state, scrollbar_state)
}

impl From<Vec<Line<'static>>> for PopupContent {
    fn from(lines: Vec<Line<'static>>) -> Self {
        let (state, scrollbar_state) = initialize_states(&lines);

        PopupContent::Select(state, scrollbar_state, lines, None)
    }
}

impl From<(Vec<Line<'static>>, Paragraph<'static>)> for PopupContent {
    fn from((lines, paragraph): (Vec<Line<'static>>, Paragraph<'static>)) -> Self {
        let (state, scrollbar_state) = initialize_states(&lines);

        PopupContent::Select(state, scrollbar_state, lines, Some(paragraph))
    }
}

impl From<String> for PopupContent {
    fn from(input: String) -> Self {
        PopupContent::Text(Line::from(input))
    }
}

impl From<&str> for PopupContent {
    fn from(input: &str) -> Self {
        PopupContent::Text(Line::from(input.to_string()))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum PopupSize {
    #[default]
    Small,
    Medium,
    Large,
}

#[derive(Debug, Default, Clone, Builder)]
#[builder(setter(into))]
pub struct Popup<'a> {
    #[builder(setter(into, strip_option), default)]
    title: Option<Line<'a>>,
    #[builder(setter(into))]
    content: PopupContent,
    #[builder(default)]
    border_style: Style,
    #[builder(default)]
    title_style: Style,
    #[builder(default)]
    popup_type: PopupType,
    #[builder(default)]
    pub(crate) size: PopupSize,
}

impl PopupBuilder<'_> {
    pub fn success(&mut self) -> &mut Self {
        self.border_style(Style::default().fg(style::Color::Green))
            .title_style(Style::default().fg(style::Color::Green).bold());
        self
    }
    pub fn error(&mut self) -> &mut Self {
        self.border_style(Style::default().fg(style::Color::Red))
            .title_style(Style::default().fg(style::Color::Red).bold());
        self
    }

    pub fn text<I: Into<Text<'static>>>(text: I) -> Self {
        let line = Paragraph::new(text).wrap(Wrap { trim: true }).centered();

        let mut builder = Self::default();
        builder.content(line);
        builder
    }

    pub fn input<I: Into<String>>(&mut self, input: I) -> &mut Self {
        let input = InputBuilder::default()
            .selected()
            .with_value(input)
            .build()
            .unwrap();

        self.content(PopupContent::Input(input));
        self
    }

    pub fn input_title<I: Into<Line<'static>>, T: Into<String>>(
        &mut self,
        title: I,
        input: T,
    ) -> &mut Self {
        let input = InputBuilder::default()
            .title(title)
            .selected()
            .with_value(input)
            .build()
            .unwrap();

        self.content(PopupContent::Input(input));
        self
    }
}

impl Popup<'_> {
    pub fn handle_key(&mut self, ev: &KeyEvent) -> Option<PopupResult> {
        match ev.code {
            KeyCode::Enter if self.popup_type == PopupType::Ok && self.is_select() => {
                if let PopupContent::Select(state, _, _, _) = &self.content
                    && let Some(idx) = state.selected() {
                        return Some(PopupResult::OkSelect(idx));
                    }
                None
            }
            KeyCode::Enter if self.popup_type == PopupType::Ok => {
                if self.is_select() {
                    if let PopupContent::Select(state, _, _, _) = &self.content
                        && let Some(idx) = state.selected() {
                            return Some(PopupResult::OkSelect(idx));
                        }
                    None
                } else if self.is_input() {
                    if let PopupContent::Input(input) = &self.content {
                        return Some(PopupResult::OkInput(input.value().to_string()));
                    }
                    None
                } else {
                    Some(PopupResult::Ok)
                }
            }
            KeyCode::Char('y') if self.popup_type == PopupType::YesNo => Some(PopupResult::Yes),
            KeyCode::Char('n') if self.popup_type == PopupType::YesNo => Some(PopupResult::No),
            _ => {
                if let PopupContent::Input(input) = &mut self.content {
                    input.handle_key(ev);
                }
                None
            }
        }
    }

    pub fn handle_control(&mut self, control: PopupControl) {
        if let PopupContent::Select(state, scrollbar_state, items, _) = &mut self.content {
            match control {
                PopupControl::Next => {
                    let current = state.selected();
                    if let Some(idx) = current
                        && idx + 1 >= items.len() {
                            return;
                        }

                    state.select_next();
                    scrollbar_state.next();
                }
                PopupControl::Previous => {
                    state.select_previous();
                    scrollbar_state.prev();
                }
                _ => {}
            }
        }
        if let PopupContent::Input(input) = &mut self.content
            && let PopupControl::SetError(err) = control {
                input.set_error(Some(&err));
            }
    }

    fn is_select(&self) -> bool {
        matches!(self.content, PopupContent::Select(_, _, _, _))
    }

    fn is_input(&self) -> bool {
        matches!(self.content, PopupContent::Input(_))
    }
}

impl Widget for &mut Popup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);

        let mut block = Block::new()
            .title_style(self.title_style)
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(self.border_style);

        if let Some(title) = &self.title {
            block = block.title(title.clone());
        }

        let inner = block.inner(area);

        block.render(area, buf);

        let layout = Layout::default()
            .direction(layout::Direction::Vertical)
            .constraints(vec![Constraint::Fill(1), Constraint::Length(1)])
            .split(inner);

        match &mut self.content {
            PopupContent::Text(line) => Paragraph::new(line.clone())
                .centered()
                .wrap(Wrap { trim: true })
                .render(layout[0], buf),
            PopupContent::Paragraph(paragraph) => paragraph.clone().render(layout[0], buf),
            PopupContent::Select(list_state, scrollbar_state, lines, paragraph) => {
                let area = match paragraph {
                    Some(paragraph) => {
                        let layout = Layout::default()
                            .direction(layout::Direction::Vertical)
                            .constraints(vec![
                                Constraint::Min(1),
                                Constraint::Max(lines.len() as u16),
                            ])
                            .split(layout[0]);
                        paragraph.clone().render(layout[0], buf);
                        layout[1]
                    }
                    None => layout[0],
                };

                let list = style_list(List::new(
                    lines
                        .iter_mut()
                        .map(|line| ratatui::widgets::ListItem::new(line.clone()))
                        .collect::<Vec<ratatui::widgets::ListItem>>(),
                ));
                StatefulWidget::render(list, area, buf, &mut list_state.clone());

                let scrollbar = style_scrollbar(Scrollbar::default());
                StatefulWidget::render(scrollbar, area, buf, &mut scrollbar_state.clone());
            }
            PopupContent::Input(input) => {
                let paragraph = input.render();
                paragraph.render(layout[0], buf);
            }
        }

        match self.popup_type {
            PopupType::Ok => {
                Paragraph::new("OK [Enter]")
                    .centered()
                    .style(Style::default().fg(style::Color::Yellow).bold())
                    .render(layout[1], buf);
            }
            PopupType::YesNo => {
                let layout = Layout::default()
                    .direction(layout::Direction::Horizontal)
                    .constraints(vec![Constraint::Fill(1), Constraint::Fill(1)])
                    .split(layout[1]);

                Paragraph::new("Yes [Y]")
                    .centered()
                    .style(Style::default().fg(style::Color::Green).bold())
                    .render(layout[0], buf);

                Paragraph::new("No [N]")
                    .centered()
                    .style(Style::default().fg(style::Color::Red).bold())
                    .render(layout[1], buf);
            }
        }
    }
}

pub enum PopupControl {
    Next,
    Previous,
    SetError(String),
}
