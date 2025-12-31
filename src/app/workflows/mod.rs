pub(crate) mod utils;

use std::path::PathBuf;

use crate::app::{
    api::{
        Api,
        structs::{Item, Platform, Version},
    },
    components::log::{LogBuilder, LogState},
    config::InstalledVersion,
    screens::main::{components::run::UpdateType, message::MainMessage},
    workflows::utils::{download_archive, extract, get_download_link},
};
use async_tempfile::TempFile;
use tokio::sync::mpsc::Sender;

pub async fn install_full_version(
    tx: Sender<MainMessage>,
    api: Api,
    path: String,
    platform: Platform,
    version: Version,
    patch: Item,
) {
    let main_log = LogBuilder::text(format!(
        "Downloading Factorio{} v{} for {}...",
        match &version {
            Version::Vanilla => "",
            Version::SpaceAge => " Space Age",
            Version::Headless => " Headless",
        },
        patch.to_string(),
        platform.to_string()
    ))
    .state(LogState::default())
    .build()
    .unwrap();
    let main_state = main_log.state.clone();
    let _ = tx.send(MainMessage::CreateLog(main_log)).await;

    let res = match get_download_link(tx.clone(), &api, &version, &platform, &patch).await {
        Some(res) => res,
        None => {
            return;
        }
    };

    let file: TempFile = match download_archive(tx.clone(), res).await {
        Some(file) => file,
        None => {
            return;
        }
    };

    match extract(tx.clone(), file, &platform, &path).await {
        Some(_) => {}
        None => {
            return;
        }
    }

    main_state.lock().unwrap().finish();

    let _ = tx
        .send(MainMessage::VersionInstalled(InstalledVersion {
            version: version,
            platform: platform,
            current_version: patch.to_string_raw().to_string(),
            path: PathBuf::from(path),
            installed_at: chrono::Utc::now(),
        }))
        .await;
}

pub async fn install_update(tx: Sender<MainMessage>, api: Api, update: UpdateType) {
    //TODO
}
