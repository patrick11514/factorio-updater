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
            main_state.lock().unwrap().error();
            return;
        }
    };

    let file: TempFile = match download_archive(tx.clone(), res).await {
        Some(file) => file,
        None => {
            main_state.lock().unwrap().error();
            return;
        }
    };

    match extract(tx.clone(), file, &platform, &path).await {
        Some(_) => {}
        None => {
            main_state.lock().unwrap().error();
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

pub async fn install_update(
    tx: Sender<MainMessage>,
    api: Api,
    uuid: uuid::Uuid,
    data: InstalledVersion,
    update: UpdateType,
) {
    let target_version = match &update {
        UpdateType::FullGame(stable) | UpdateType::FullGameUnsupported(stable) => {
            stable.to_string()
        }
        UpdateType::Patch(diffs) => diffs.last().unwrap().to.to_string(),
    };

    let main_log = LogBuilder::text(format!(
        "Updating Factorio{} {} from {} to {}...",
        match &data.version {
            Version::Vanilla => "",
            Version::SpaceAge => " Space Age",
            Version::Headless => " Headless",
        },
        data.platform.to_string(),
        data.current_version,
        target_version,
    ))
    .state(LogState::default())
    .build()
    .unwrap();
    let main_state = main_log.state.clone();
    let _ = tx.send(MainMessage::CreateLog(main_log)).await;

    //if -> patch, try to download patches, otherwise we fallback to full download
    if let UpdateType::Patch(version_diffs) = &update {
        let log = LogBuilder::text("Downloading patches...")
            .state(LogState::default())
            .build()
            .unwrap();
        let state = log.state.clone();
        let _ = tx.send(MainMessage::CreateLog(log)).await;

        let log_progress = LogBuilder::progress(0)
            .state(LogState::default())
            .build()
            .unwrap();
        let progress_state = log_progress.state.clone();
        //let progress_fac
        let _ = tx.send(MainMessage::CreateLog(log_progress)).await;
    }

    main_state.lock().unwrap().finish();
    let _ = tx
        .send(MainMessage::VersionUpdated(uuid, target_version))
        .await;
}
