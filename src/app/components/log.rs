use derive_builder::Builder;
use ratatui::{
    layout::Rect,
    style::{self, Style},
    text::Line,
    widgets::ListItem,
};

static TICK_STRINGS: &[&'static str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
static TICK_SLOW: usize = 4;

#[derive(Clone, Debug)]
pub enum LogState {
    Finished(chrono::Duration),
    Errored(chrono::Duration),
    InProgress {
        started_at: chrono::DateTime<chrono::Local>,
        prev_char_idx: usize,
    },
}

impl Default for LogState {
    fn default() -> Self {
        LogState::InProgress {
            started_at: chrono::Local::now(),
            prev_char_idx: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum LogType {
    Text(String),
    Progress(u8),
}

#[derive(Clone, Debug, Builder)]
#[builder(setter(into))]
pub struct Log {
    #[builder(default)]
    pub state: LogState,
    pub log_type: LogType,
    #[builder(default = "chrono::Local::now()")]
    pub timestamp: chrono::DateTime<chrono::Local>,
}

impl Log {
    pub fn render(&mut self, area: &Rect) -> ListItem<'static> {
        let state_symbol = match &mut self.state {
            LogState::Finished(_) => "✓",
            LogState::Errored(_) => "✗",
            LogState::InProgress { prev_char_idx, .. } => {
                let char = TICK_STRINGS[*prev_char_idx / TICK_SLOW];
                *prev_char_idx = (*prev_char_idx + 1) % (TICK_STRINGS.len() * TICK_SLOW);

                char
            }
        };

        let style = match self.state {
            LogState::Finished(_) => Style::default().fg(style::Color::Green),
            LogState::Errored(_) => Style::default().fg(style::Color::Red),
            LogState::InProgress { .. } => Style::default().fg(style::Color::Yellow),
        };

        let timestamp = self.timestamp.format("%Y-%m-%d %H:%M:%S");

        let content = match &self.log_type {
            LogType::Text(text) => text.clone(),
            LogType::Progress(progress) => {
                let bar_length = (area.width as usize)
                    .saturating_sub(21 /* timestamp */ + 3 /* symbol */ + 7 /* percentage*/);
                let filled_length = (*progress as usize * bar_length) / 100;
                let bar = format!(
                    "[{}O{}] {}%",
                    "=".repeat(filled_length.saturating_sub(1)),
                    " ".repeat(bar_length - filled_length),
                    progress
                );

                bar
            }
        };

        let content =
            Line::from(format!("[{}] {} {}", timestamp, state_symbol, content)).style(style);

        ListItem::new(content)
    }

    pub fn to_errored(&mut self) {
        let duration = chrono::Local::now() - self.timestamp;
        self.state = LogState::Errored(duration);
    }

    pub fn to_finished(&mut self) {
        let duration = chrono::Local::now() - self.timestamp;
        self.state = LogState::Finished(duration);
    }

    pub fn is_in_progress(&self) -> bool {
        matches!(self.state, LogState::InProgress { .. })
    }

    pub fn is_finished(&self) -> bool {
        matches!(self.state, LogState::Finished(_))
    }
}
