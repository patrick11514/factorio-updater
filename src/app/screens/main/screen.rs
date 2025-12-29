use async_trait::async_trait;
use crossterm::event::KeyEvent;
use ratatui::text::Line;
use tokio::{sync::mpsc, task::JoinHandle};

use crate::app::{
    api::Api,
    components::{
        log::{Log, LogBuilder, LogState},
        popup::{PopupBuilder, PopupResult},
    },
    screens::{
        Screen, ScreenEvent,
        main::{
            components::{
                render::render,
                run::check_credentials,
                tick::{OpenedPopup, tick},
            },
            message::MainMessage,
        },
    },
};

#[derive(PartialEq, Default, Debug, Clone)]
pub enum RunState {
    #[default]
    CheckingCredentials,
    FetchingVersions,
    Idle,
}

pub struct Main {
    pub(crate) username: String,
    pub(crate) api: Api,
    pub(crate) logs: Vec<Box<Log>>,
    pub(crate) rx: mpsc::Receiver<MainMessage>,
    pub(crate) tx: mpsc::Sender<MainMessage>,
    pub(crate) opened_popup: Option<OpenedPopup>,
    pub(crate) selected_version: Option<usize>,
    pub(crate) run_state: RunState,
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
        }
    }
}

#[async_trait]
impl Screen for Main {
    fn run(&mut self) -> Option<JoinHandle<()>> {
        let tx = self.tx.clone();
        let api = self.api.clone();

        let state = self.run_state.clone();

        Some(tokio::spawn(async move {
            //TODO match and then start specific tasks

            check_credentials(&api, &tx).await;
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
                        self.run();

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
