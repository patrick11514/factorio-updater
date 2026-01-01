use std::{collections::HashMap, env, fmt::Display, sync::Arc};

use tokio::sync::mpsc::Sender;

use crate::app::{
    api::{
        Api,
        structs::{Arch, Item, Platform, Updates, VersionDiff},
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
    let _ = tx.send(MainMessage::CreateLog(log)).await;

    match api.check_credentials().await {
        Ok(creds) => match creds {
            true => {
                let _ = tx.send(MainMessage::CheckLogin(Ok(Some(())), state)).await;
                return Some(RunState::FetchingVersions);
            }
            false => {
                let _ = tx.send(MainMessage::CheckLogin(Ok(None), state)).await;
                return None;
            }
        },
        Err(err) => {
            let _ = tx.send(MainMessage::CheckLogin(Err(err), state)).await;
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
    let _ = tx.send(MainMessage::CreateLog(log)).await;

    let result = api.get_patches().await;

    let next_state = match &result {
        Ok(_) => Some(RunState::Idle),
        Err(_) => None,
    };

    let _ = tx.send(MainMessage::LoadVersions(result, state)).await;

    next_state
}

#[derive(Debug, Clone)]
pub enum UpdateType {
    FullGame(String),
    FullGameUnsupported(String),
    Patch(Vec<VersionDiff>),
}

impl Display for UpdateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateType::FullGame(version) => write!(f, "Full Game to version {}", version),
            UpdateType::Patch(diffs) => write!(f, "Patch with {} updates", diffs.len()),
            UpdateType::FullGameUnsupported(version) => {
                write!(f, "Full Game to version {} (unsupported OS)", version)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum InstalledVersionState {
    UpToDate,
    Updating,
    Updated,
    UpdateAvailable(UpdateType),
}

#[derive(Debug, Clone)]
pub struct InstalledVersionDetails {
    pub(crate) state: InstalledVersionState,
}

pub async fn check_for_updates(
    updates: Arc<Updates>,
    installed_versions: &HashMap<uuid::Uuid, InstalledVersion>,
    current_details: &HashMap<uuid::Uuid, InstalledVersionDetails>,
    tx: &Sender<MainMessage>,
) -> Option<RunState> {
    let log = LogBuilder::text("Checking for updates...")
        .state(LogState::default())
        .build()
        .unwrap();

    let state = log.state.clone();
    let _ = tx.send(MainMessage::CreateLog(log)).await;

    let version_details = installed_versions
        .iter()
        .map(|(uuid, iv)| {
            if current_details.len() > 0 {
                if let Some(details) = current_details.get(uuid) {
                    if let InstalledVersionState::Updating = details.state {
                        //if some version is updating, we don't wan't to overwrite its state
                        return (uuid.clone(), details.clone());
                    }
                }
            }

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
                //Check os compatibility

                let is_supported_os = match (env::consts::OS, &iv.platform) {
                    ("linux", Platform::Linux64 | Platform::Linux32) => true,
                    ("windows", Platform::Win64 | Platform::Win32) => true,
                    ("macos", Platform::Mac | Platform::MacArm64 | Platform::MacX64) => true,
                    _ => false,
                };

                let mut collected_updates = Vec::new();
                let mut to_walk = vec![&iv.current_version];

                let update_type = if !is_supported_os {
                    UpdateType::FullGameUnsupported(stable)
                } else {
                    loop {
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
                    }
                };

                InstalledVersionState::UpdateAvailable(update_type)
            };

            (uuid.clone(), InstalledVersionDetails { state })
        })
        .collect();

    let _ = tx
        .send(MainMessage::VersionDetails(version_details, state))
        .await;

    Some(RunState::Idle)
}
