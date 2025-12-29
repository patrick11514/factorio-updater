use std::sync::Arc;

use tokio::sync::mpsc::Sender;

use crate::app::{
    api::{
        Api,
        structs::{Arch, Item, Updates, VersionDiff},
    },
    components::log::{LogBuilder, LogState},
    config::InstalledVersion,
    screens::main::message::MainMessage,
};
use semver::Version;

#[derive(Default, Debug, Clone)]
pub enum RunState {
    #[default]
    CheckingCredentials,
    FetchingVersions,
    CheckForUpdates,
    Idle,
}

pub async fn check_credentials(api: &Api, tx: &Sender<MainMessage>) -> Option<RunState> {
    let log = LogBuilder::text("Checking login...")
        .state(LogState::default())
        .build()
        .unwrap();

    let state = log.state.clone();
    tx.send(MainMessage::CreateLog(log)).await.unwrap();

    match api.check_credentials().await {
        Ok(creds) => match creds {
            true => {
                tx.send(MainMessage::CheckLogin(Ok(Some(())), state))
                    .await
                    .unwrap();
                return Some(RunState::FetchingVersions);
            }
            false => {
                tx.send(MainMessage::CheckLogin(Ok(None), state))
                    .await
                    .unwrap();
                return None;
            }
        },
        Err(err) => {
            tx.send(MainMessage::CheckLogin(Err(err), state))
                .await
                .unwrap();
            return None;
        }
    };
}

pub async fn fetch_versions(api: &Api, tx: &Sender<MainMessage>) -> Option<RunState> {
    let log = LogBuilder::text("Fetching available versions...")
        .state(LogState::default())
        .build()
        .unwrap();

    let state = log.state.clone();
    tx.send(MainMessage::CreateLog(log)).await.unwrap();

    let result = api.get_versions().await;

    let next_state = match &result {
        Ok(_) => Some(RunState::Idle),
        Err(_) => None,
    };

    tx.send(MainMessage::LoadVersions(result, state))
        .await
        .unwrap();

    next_state
}

#[derive(Debug, Clone)]
pub enum UpdateType {
    FullGame(String),
    Patch(Vec<VersionDiff>),
}

#[derive(Debug, Clone)]
pub enum InstalledVersionState {
    UpToDate,
    UpdateAvailable(UpdateType),
}

#[derive(Debug, Clone)]
pub struct InstalledVersionDetails {
    pub(crate) state: InstalledVersionState,
}

pub async fn check_for_updates(
    updates: Arc<Updates>,
    installed_versions: &Vec<InstalledVersion>,
    tx: &Sender<MainMessage>,
) -> Option<RunState> {
    let log = LogBuilder::text("Checking for updates...")
        .state(LogState::default())
        .build()
        .unwrap();

    let state = log.state.clone();
    tx.send(MainMessage::CreateLog(log)).await.unwrap();

    let version_details = installed_versions
        .into_iter()
        .map(|iv| {
            let arch: Arch = (&iv.version, &iv.platform).into();
            let arch_updates = updates.get(&arch).unwrap();

            let stable = arch_updates
                .iter()
                .find(|item| matches!(item, Item::Stable(_)))
                .and_then(|item| {
                    if let Item::Stable(stable) = item {
                        Some(stable.stable.clone())
                    } else {
                        None
                    }
                })
                .unwrap();

            let state = if Version::parse(&iv.current_version).unwrap()
                == Version::parse(&stable).unwrap()
            {
                InstalledVersionState::UpToDate
            } else {
                let mut collected_updates = Vec::new();
                let mut to_walk = vec![&iv.current_version];

                let update_type = loop {
                    let version = if let Some(version) = to_walk.pop() {
                        version
                    } else {
                        break UpdateType::FullGame(stable);
                    };

                    match arch_updates.iter().find(|item| match item {
                        Item::VersionDiff(version_diff) => version_diff.from == *version,
                        Item::Stable(stable) => stable.stable == *version,
                    }) {
                        Some(item) => match item {
                            Item::VersionDiff(version_diff) => {
                                //accumulate path for patching
                                collected_updates.push(version_diff.clone());
                                to_walk.push(&version_diff.to);
                            }
                            //we reached stable version, so stop here
                            Item::Stable(_) => {
                                break UpdateType::Patch(collected_updates);
                            }
                        },
                        None => {
                            //Somehow we can't find a diff
                            break UpdateType::FullGame(stable);
                        }
                    }
                };

                InstalledVersionState::UpdateAvailable(update_type)
            };

            InstalledVersionDetails { state }
        })
        .collect();

    tx.send(MainMessage::VersionDetails(version_details, state))
        .await
        .unwrap();

    Some(RunState::Idle)
}
