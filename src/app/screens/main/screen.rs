use std::{pin::Pin, sync::Arc, thread::current};

use async_trait::async_trait;
use crossterm::event::KeyEvent;
use ratatui::text::Line;
use tokio::{sync::mpsc, task::JoinHandle};

use crate::app::{
    api::{
        Api,
        structs::{Updates, Version},
    },
    components::{
        log::{Log, LogBuilder, LogState},
        popup::{PopupBuilder, PopupResult},
    },
    screens::{
        Screen, ScreenEvent,
        main::{
            components::{
                render::render,
                run::{
                    InstalledVersionDetails, RunState, check_credentials, check_for_updates,
                    fetch_versions,
                },
                tick::{OpenedPopup, tick},
            },
            message::MainMessage,
        },
    },
};

pub struct Main {
    pub(crate) username: String,
    pub(crate) api: Api,
    pub(crate) logs: Vec<Box<Log>>,
    pub(crate) rx: mpsc::Receiver<MainMessage>,
    pub(crate) tx: mpsc::Sender<MainMessage>,
    pub(crate) opened_popup: Option<OpenedPopup>,
    pub(crate) selected_version: Option<usize>,
    pub(crate) run_state: RunState,
    pub(crate) updates: Option<Arc<Updates>>,
    pub(crate) installed_version_details: Vec<InstalledVersionDetails>,
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
            selected_version: None,
            run_state: Default::default(),
            installed_version_details: Vec::new(),
            updates: None,
        }
    }
}

#[async_trait]
impl Screen for Main {
    fn init(&mut self) -> Option<JoinHandle<()>> {
        let tx = self.tx.clone();
        let api = self.api.clone();

        let state = self.run_state.clone();
        let updates = self.updates.clone();

        Some(tokio::spawn(async move {
            let mut current_state = state;
            loop {
                let next_step = match current_state {
                    RunState::CheckingCredentials => check_credentials(&api, &tx).await,
                    RunState::FetchingVersions => fetch_versions(&api, &tx).await,
                    RunState::CheckForUpdates => {
                        check_for_updates(
                            updates.clone().unwrap(),
                            &api.config.installed_versions,
                            &tx,
                        )
                        .await
                    }
                    RunState::Idle => return,
                };

                match next_step {
                    None => {
                        return;
                    }
                    Some(next_state) => {
                        current_state = next_state;
                        tx.send(MainMessage::ChangeRunState(current_state.clone()))
                            .await
                            .unwrap();
                    }
                }
            }
        }))
    }

    fn tick(&mut self) -> Option<ScreenEvent> {
        tick(self)
    }

    fn render(&mut self, frame: &mut ratatui::Frame) {
        render(self, frame);
    }

    async fn on_key(&mut self, _: &KeyEvent) -> Option<ScreenEvent> {
        None
    }

    async fn on_popup(&mut self, res: PopupResult) -> Option<ScreenEvent> {
        if let Some(opened) = &self.opened_popup {
            match res {
                PopupResult::Ok => match opened {
                    OpenedPopup::LogoutNotify => Some(ScreenEvent::Logout),
                    OpenedPopup::ErrorNotify => {
                        self.opened_popup = None;

                        // Retry checking credentials
                        self.init();

                        Some(ScreenEvent::ClosePopup)
                    }
                },
                _ => None,
            }
        } else {
            None
        }
    }
}
