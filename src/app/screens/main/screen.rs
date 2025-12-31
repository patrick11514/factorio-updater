use std::{collections::HashMap, hash::Hash, sync::Arc};

use async_trait::async_trait;
use crossterm::event::KeyEvent;
use ratatui::widgets::{ListState, ScrollbarState};
use tokio::{sync::mpsc, task::JoinHandle};

use crate::app::{
    api::{Api, structs::Updates},
    components::{log::Log, popup::PopupResult},
    config::InstalledVersion,
    screens::{
        Screen, ScreenEvent,
        main::{
            components::{
                on_key::on_key,
                on_popup::on_popup,
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

#[derive(Debug, Clone, Default)]
pub enum SelectedList {
    #[default]
    Versions,
    Logs,
}

pub struct Main {
    pub(crate) username: String,
    pub(crate) api: Api,
    pub(crate) logs: Vec<Log>,

    pub(crate) rx: mpsc::Receiver<MainMessage>,
    pub(crate) tx: mpsc::Sender<MainMessage>,

    pub(crate) opened_popup: Option<OpenedPopup>,
    pub(crate) run_state: RunState,

    pub(crate) updates: Option<Arc<Updates>>,
    pub(crate) installed_version_details: HashMap<uuid::Uuid, InstalledVersionDetails>,

    pub(crate) selected_version: Option<uuid::Uuid>,
    pub(crate) selected_log: Option<usize>,

    pub(crate) selected_list: SelectedList,

    pub(crate) version_list_state: ListState,
    pub(crate) version_scrollbar_state: ScrollbarState,

    pub(crate) logs_list_state: ListState,
    pub(crate) logs_scrollbar_state: ScrollbarState,
}

impl Main {
    pub fn new(api: Api) -> Self {
        let (tx, rx) = mpsc::channel(128);

        let mut selected_version = None;
        let mut version_list_state = ListState::default();
        let mut version_scrollbar_state = ScrollbarState::default();

        if !api.config.installed_versions.is_empty() {
            selected_version = Some(api.config.installed_versions.keys().next().unwrap().clone());

            version_list_state.select(Some(0));
            version_scrollbar_state =
                version_scrollbar_state.content_length(api.config.installed_versions.len());
        }

        Self {
            username: api.config.username.clone(),
            api,
            logs: Vec::new(),
            rx,
            tx,
            opened_popup: None,
            selected_version,
            selected_log: None,
            run_state: Default::default(),
            installed_version_details: HashMap::new(),
            updates: None,
            version_list_state,
            version_scrollbar_state,
            selected_list: Default::default(),
            logs_list_state: ListState::default(),
            logs_scrollbar_state: ScrollbarState::default(),
        }
    }

    pub fn get_installed_versions(
        versions: &HashMap<uuid::Uuid, InstalledVersion>,
    ) -> Vec<(&uuid::Uuid, &InstalledVersion)> {
        let mut versions: Vec<(&uuid::Uuid, &InstalledVersion)> = versions.iter().collect();

        versions.sort_by_key(|v| v.1.installed_at);
        versions
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
                        let _ = tx
                            .send(MainMessage::ChangeRunState(current_state.clone()))
                            .await;
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

    async fn on_key(&mut self, ev: &KeyEvent) -> Option<ScreenEvent> {
        on_key(self, ev).await
    }

    async fn on_popup(&mut self, res: PopupResult) -> Option<ScreenEvent> {
        on_popup(self, res).await
    }
}
