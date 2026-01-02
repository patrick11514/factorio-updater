#![allow(dead_code)]

use std::sync::{
    Arc, Mutex,
    atomic::{self, AtomicU8},
};

use chrono::Duration;
use derive_builder::Builder;
use ratatui::{
    layout::Rect,
    style::{self, Style},
    text::{Line, Span},
    widgets::ListItem,
};

use crate::app::utils::ORANGE;

static TICK_STRINGS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
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

impl LogState {
    pub fn finish(&mut self) {
        if let LogState::InProgress { started_at, .. } = *self {
            let duration = chrono::Local::now() - started_at;
            *self = LogState::Finished(duration);
        }
    }

    pub fn error(&mut self) {
        if let LogState::InProgress { started_at, .. } = *self {
            let duration = chrono::Local::now() - started_at;
            *self = LogState::Errored(duration);
        }
    }
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
    Progress(Arc<AtomicU8>),
}

#[derive(Clone, Debug, Builder)]
#[builder(setter(into))]
pub struct Log {
    #[builder(setter(custom))]
    pub state: Arc<Mutex<LogState>>,
    pub log_type: LogType,
    #[builder(default = "chrono::Local::now()")]
    pub timestamp: chrono::DateTime<chrono::Local>,
}

impl LogBuilder {
    pub fn state(&mut self, state: LogState) -> &mut Self {
        self.state = Some(Arc::new(Mutex::new(state)));
        self
    }

    pub fn text<T: Into<String>>(text: T) -> Self {
        let mut builder = LogBuilder::default();
        builder.log_type(LogType::Text(text.into()));
        builder
    }

    pub fn progress(progress: u8) -> Self {
        let mut builder = LogBuilder::default();
        builder.log_type(LogType::Progress(Arc::new(AtomicU8::new(progress))));
        builder
    }
}

fn get_highest_duration(duration: Duration) -> String {
    if duration.num_hours() > 0 {
        format!("{}h", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{}m", duration.num_minutes())
    } else if duration.num_seconds() > 0 {
        format!("{}s", duration.num_seconds())
    } else {
        format!("{}ms", duration.num_milliseconds())
    }
}

impl Log {
    pub fn render(&mut self, area: &Rect, selected: bool) -> ListItem<'static> {
        let mut state = self.state.lock().unwrap();

        let style = match *state {
            LogState::Finished(_) => Style::default().fg(style::Color::Green),
            LogState::Errored(_) => Style::default().fg(style::Color::Red),
            LogState::InProgress { .. } => Style::default().fg(style::Color::Yellow),
        };

        let state_symbol = Span::styled(
            match *state {
                LogState::Finished(dur) => format!("✓ ({})", get_highest_duration(dur)),
                LogState::Errored(dur) => format!("✗ ({})", get_highest_duration(dur)),
                LogState::InProgress {
                    ref mut prev_char_idx,
                    ..
                } => {
                    let char = TICK_STRINGS[*prev_char_idx / TICK_SLOW];
                    *prev_char_idx = (*prev_char_idx + 1) % (TICK_STRINGS.len() * TICK_SLOW);

                    char.to_string()
                }
            },
            style,
        );
        let timestamp = self.timestamp.format("%Y-%m-%d %H:%M:%S");

        let content = match &self.log_type {
            LogType::Text(text) => Span::styled(text.clone(), style),
            LogType::Progress(progress) => {
                let progress = progress.load(atomic::Ordering::Relaxed).min(100);

                let bar_length = (area.width as usize)
                    .saturating_sub(21 /* timestamp */ + 2 + state_symbol.content.len() /* symbol */ + 7 /* percentage*/ + if selected {2} else {0} /* select */);
                let filled_length = (progress as usize * bar_length) / 100;
                let bar = format!(
                    "[{}O{}] {}%",
                    "=".repeat(filled_length.saturating_sub(1)),
                    " ".repeat(bar_length.saturating_sub(filled_length)),
                    progress
                );

                Span::styled(
                    bar,
                    Style::default().fg(match progress {
                        0..=15 => style::Color::Indexed(196), /* Red */
                        16..=30 => ORANGE,
                        31..=45 => style::Color::Indexed(208), /* Orange */
                        46..=60 => style::Color::Indexed(214), /* Yellow */
                        61..=75 => style::Color::Indexed(226), /* Yellow-Green */
                        76..=90 => style::Color::Indexed(118), /* Light Green */
                        91..=100 => style::Color::Indexed(46), /* Green */
                        _ => style::Color::White,
                    }),
                )
            }
        };

        let content = Line::from(vec![
            Span::from(format!("[{}] ", timestamp)),
            state_symbol,
            Span::from(" "),
            content,
        ]);

        ListItem::new(content)
    }

    pub fn get_progress(&self) -> Arc<AtomicU8> {
        if let LogType::Progress(ref progress) = self.log_type {
            progress.clone()
        } else {
            Arc::new(AtomicU8::new(0))
        }
    }
}
