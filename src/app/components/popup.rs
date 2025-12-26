use crossterm::event::{KeyCode, KeyEvent};
use derive_builder::Builder;
use ratatui::{
    buffer::Buffer,
    layout::{self, Constraint, Layout, Rect},
    style::{self, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget, Wrap},
};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum PopupType {
    #[default]
    Ok,
    YesNo,
}

#[derive(Debug, Clone, Copy)]
pub enum PopupResult {
    Ok,
    Yes,
    No,
}

#[derive(Debug, Default, Clone, Builder)]
#[builder(setter(into))]
pub struct Popup<'a> {
    #[builder(setter(into, strip_option), default)]
    title: Option<Line<'a>>,
    content: Text<'a>,
    #[builder(default)]
    border_style: Style,
    #[builder(default)]
    title_style: Style,
    #[builder(default)]
    style: Style,
    #[builder(default)]
    popup_type: PopupType,
}

impl Popup<'_> {
    pub fn handle_key(&mut self, ev: &KeyEvent) -> Option<PopupResult> {
        match ev.code {
            KeyCode::Enter if self.popup_type == PopupType::Ok => Some(PopupResult::Ok),
            KeyCode::Char('y') if self.popup_type == PopupType::YesNo => Some(PopupResult::Yes),
            KeyCode::Char('n') if self.popup_type == PopupType::YesNo => Some(PopupResult::No),
            _ => None,
        }
    }
}

impl Widget for Popup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);

        let mut block = Block::new()
            .title_style(self.title_style)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(self.border_style);

        if let Some(title) = self.title {
            block = block.title(title);
        }

        let inner = block.inner(area);

        block.render(area, buf);

        let layout = Layout::default()
            .direction(layout::Direction::Vertical)
            .constraints(vec![Constraint::Fill(1), Constraint::Length(1)])
            .split(inner);

        Paragraph::new(self.content)
            .wrap(Wrap { trim: true })
            .centered()
            .style(self.style)
            .render(layout[0], buf);

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
