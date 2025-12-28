use async_trait::async_trait;
use crossterm::event::KeyEvent;
use derive_builder::Builder;
use ratatui::{
    layout::{self, Layout},
    style::{self, Color, Style, Styled, Stylize},
    symbols::{border, merge::MergeStrategy},
    text::Line,
    widgets::{Block, BorderType, List, ListDirection, ListItem, Paragraph},
};

use crate::{
    app::{
        api::Api,
        components::{
            log::{Log, LogBuilder, LogState, LogType},
            popup::PopupResult,
        },
        screens::{Screen, ScreenEvent},
    },
    utils::{border_with_title, with_title},
};

pub struct Main {
    username: String,
    api: Api,
    logs: Vec<Box<Log>>,
}

impl Main {
    pub fn new(api: Api) -> Self {
        Self {
            username: api.config.username.clone(),
            api,
            logs: vec![
                Box::new(
                    LogBuilder::default()
                        .log_type(LogType::Text("Starting updater...".into()))
                        .state(LogState::Finished(chrono::Duration::zero()))
                        .build()
                        .unwrap(),
                ),
                Box::new(
                    LogBuilder::default()
                        .log_type(LogType::Text("Checking for updates...".into()))
                        .state(LogState::default())
                        .build()
                        .unwrap(),
                ),
                Box::new(
                    LogBuilder::default()
                        .log_type(LogType::Progress(45))
                        .state(LogState::default())
                        .build()
                        .unwrap(),
                ),
                Box::new(
                    LogBuilder::default()
                        .log_type(LogType::Progress(100))
                        .state(LogState::Finished(chrono::Duration::seconds(5)))
                        .build()
                        .unwrap(),
                ),
                Box::new(
                    LogBuilder::default()
                        .log_type(LogType::Text("Update completed successfully.".into()))
                        .state(LogState::Errored(chrono::Duration::zero()))
                        .build()
                        .unwrap(),
                ),
            ],
        }
    }
}

#[async_trait]
impl Screen for Main {
    fn render(&mut self, frame: &mut ratatui::Frame) {
        let outer = Block::default();

        let inner = outer.inner(frame.area());

        let layout = Layout::default()
            .spacing(layout::Spacing::Overlap(1))
            .direction(layout::Direction::Vertical)
            .constraints(vec![
                layout::Constraint::Length(3),
                layout::Constraint::Ratio(2, 3),
                layout::Constraint::Min(3),
            ])
            .split(inner);

        frame.render_widget(outer, frame.area());

        let title = Paragraph::new(
            Line::from(format!("Factorio Updater - {}", self.api.config.username))
                .style(Style::default().fg(Color::Indexed(208) /* Orange */).bold()),
        )
        .block(
            Block::bordered()
                .border_type(BorderType::Plain)
                .merge_borders(MergeStrategy::Exact),
        );

        frame.render_widget(title, layout[0]);

        let main_layout = Layout::default()
            .spacing(layout::Spacing::Overlap(1))
            .direction(layout::Direction::Horizontal)
            .constraints(vec![
                layout::Constraint::Percentage(50),
                layout::Constraint::Percentage(51), /* because of overlap -> one col at the end was empty */
            ])
            .split(layout[1]);

        let installed_versions = border_with_title(
            frame,
            Line::from("Installed Versions").style(
                Style::default()
                    .fg(Color::Indexed(112) /* Light Green */)
                    .bold(),
            ),
            main_layout[0],
        );

        let more_info = border_with_title(
            frame,
            Line::from("More Info").style(
                Style::default()
                    .fg(Color::Indexed(45) /* Light Blue */)
                    .bold(),
            ),
            main_layout[1],
        );

        let logs_container = border_with_title(
            frame,
            Line::from("Logs").style(Style::default().fg(Color::Indexed(196) /* Red */).bold()),
            layout[2],
        );

        let items = self
            .logs
            .iter_mut()
            .rev()
            .map(|log| log.render(&logs_container));

        let logs = List::new(items).direction(ListDirection::BottomToTop);

        let progress = self
            .logs
            .iter_mut()
            .find(|log| log.is_in_progress() && matches!(log.log_type, LogType::Progress(_)));

        if let Some(log) = progress {
            if let LogType::Progress(ref mut progress) = log.log_type {
                *progress = (*progress + 1).min(100);
                if *progress == 100 {
                    log.to_finished();
                }
            }
        } else {
            self.logs.push(Box::new(
                LogBuilder::default()
                    .log_type(LogType::Progress(0))
                    .state(LogState::default())
                    .build()
                    .unwrap(),
            ));
        }

        frame.render_widget(logs, logs_container);
    }

    async fn on_key(&mut self, _: &KeyEvent) -> Option<ScreenEvent> {
        None
    }

    async fn on_popup(&mut self, _: PopupResult) -> Option<ScreenEvent> {
        None
    }
}
