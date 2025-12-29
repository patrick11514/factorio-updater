use std::sync::atomic;

use async_trait::async_trait;
use crossterm::event::KeyEvent;
use derive_builder::Builder;
use ratatui::{
    layout::{self, Layout},
    style::{Color, Style, Styled, Stylize},
    symbols::merge::MergeStrategy,
    text::Line,
    widgets::{Block, BorderType, List, ListDirection, Paragraph},
};
use tokio::{sync::mpsc, task::JoinHandle};

use crate::{
    app::{
        api::Api,
        components::{
            log::{Log, LogBuilder, LogState, LogType},
            popup::{PopupBuilder, PopupResult},
        },
        screens::{Screen, ScreenEvent, main::message::MainMessage},
    },
    utils::border_with_title,
};

enum OpenedPopup {
    LogoutNotify,
}

pub struct Main {
    username: String,
    api: Api,
    logs: Vec<Box<Log>>,
    rx: mpsc::Receiver<MainMessage>,
    tx: mpsc::Sender<MainMessage>,
    opened_popup: Option<OpenedPopup>,
}

impl Main {
    pub fn new(api: Api) -> Self {
        let (tx, rx) = mpsc::channel(128);

        Self {
            username: api.config.username.clone(),
            api,
            logs: Vec::new(),
            rx,
            tx,
            opened_popup: None,
        }
    }
}

#[async_trait]
impl Screen for Main {
    fn run(&mut self) -> Option<JoinHandle<()>> {
        let tx = self.tx.clone();
        let api = self.api.clone();

        let log = Box::new(
            LogBuilder::text("Checking login...")
                .state(LogState::default())
                .build()
                .unwrap(),
        );

        let state = log.state.clone();

        self.logs.push(log);

        Some(tokio::spawn(async move {
            match api.check_credentials().await {
                Ok(creds) => match creds {
                    true => {
                        tx.send(MainMessage::CheckLogin(Ok(Some(())), state))
                            .await
                            .unwrap();
                    }
                    false => {
                        tx.send(MainMessage::CheckLogin(Ok(None), state))
                            .await
                            .unwrap();
                        return;
                    }
                },
                Err(err) => {
                    tx.send(MainMessage::CheckLogin(Err(err), state))
                        .await
                        .unwrap();
                    return;
                }
            };

            //LOAD VERS
        }))
    }

    fn tick(&mut self) -> Option<ScreenEvent> {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                MainMessage::CheckLogin(result, state) => match result {
                    Ok(Some(())) => {
                        state.lock().unwrap().finish();
                    }
                    Ok(None) => {
                        state.lock().unwrap().error();
                        self.opened_popup = Some(OpenedPopup::LogoutNotify);
                        return Some(ScreenEvent::OpenPopup(
                            PopupBuilder::error()
                                .title(Line::from("Logout").centered())
                                .content("Username or token is invalid, please login again.")
                                .build()
                                .unwrap(),
                        ));
                    }
                    Err(err) => {
                        state.lock().unwrap().error();
                    }
                },
                _ => {}
            }
        }

        None
    }

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

        frame.render_widget(logs, logs_container);
    }

    async fn on_key(&mut self, _: &KeyEvent) -> Option<ScreenEvent> {
        None
    }

    async fn on_popup(&mut self, res: PopupResult) -> Option<ScreenEvent> {
        if let Some(opened) = &self.opened_popup {
            match res {
                PopupResult::Ok => match opened {
                    OpenedPopup::LogoutNotify => Some(ScreenEvent::Logout),
                },
                _ => None,
            }
        } else {
            None
        }
    }
}
