#![allow(dead_code)]

use std::sync::{
    Arc, Mutex,
    atomic::{self, AtomicU8},
};

use derive_builder::Builder;
use ratatui::{
    layout::Rect,
    style::{self, Style},
    text::{Line, Span},
    widgets::ListItem,
};

use crate::app::utils::ORANGE;

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

impl Log {
    pub fn render(&mut self, area: &Rect) -> ListItem<'static> {
        let mut state = self.state.lock().unwrap();

        let style = match *state {
            LogState::Finished(_) => Style::default().fg(style::Color::Green),
            LogState::Errored(_) => Style::default().fg(style::Color::Red),
            LogState::InProgress { .. } => Style::default().fg(style::Color::Yellow),
        };

        let state_symbol = Span::styled(
            match *state {
                LogState::Finished(dur) => format!("✓ ({}ms)", dur.num_milliseconds()),
                LogState::Errored(dur) => format!("✗ ({}ms)", dur.num_milliseconds()),
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
                    .saturating_sub(21 /* timestamp */ + 2 + state_symbol.content.len() /* symbol */ + 7 /* percentage*/);
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
                        0..=15 => style::Color::Indexed(196),
                        16..=30 => ORANGE,
                        31..=45 => style::Color::Indexed(208),
                        46..=60 => style::Color::Indexed(214),
                        61..=75 => style::Color::Indexed(226),
                        76..=90 => style::Color::Indexed(118),
                        91..=100 => style::Color::Indexed(46),
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

    pub fn set_progresss(&self, amount: u8) {
        if let LogType::Progress(ref progress) = self.log_type {
            progress
                .fetch_update(atomic::Ordering::Relaxed, atomic::Ordering::Relaxed, |_| {
                    Some(amount.min(100))
                })
                .unwrap();
        }
    }
}
