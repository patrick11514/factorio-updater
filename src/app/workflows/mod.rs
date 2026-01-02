pub(crate) mod utils;

use std::{path::PathBuf, process::Stdio, sync::atomic::Ordering};

use crate::app::{
    api::{
        Api, ApiError,
        structs::{Item, Platform, Stable, Version},
    },
    components::{
        log::{LogBuilder, LogState},
        popup::PopupBuilder,
    },
    config::InstalledVersion,
    screens::main::{components::run::UpdateType, message::MainMessage},
    workflows::utils::{download_archive, extract, find_factorio_binary, get_download_link},
};
use async_tempfile::TempFile;
use futures_util::{
    StreamExt,
    future::{join_all, try_join_all},
};
use tokio::{io::AsyncWriteExt, process::Command, sync::mpsc::Sender};

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
        patch,
        platform
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

    let path = PathBuf::from(&path);

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
            version,
            platform,
            current_version: patch.to_string_raw().to_string(),
            path,
            installed_at: chrono::Utc::now(),
        }))
        .await;
}

enum PatchError {
    PatchNotFound,
    DownloadFailed,
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
        data.platform,
        data.current_version,
        target_version,
    ))
    .state(LogState::default())
    .build()
    .unwrap();
    let main_state = main_log.state.clone();
    let _ = tx.send(MainMessage::CreateLog(main_log)).await;

    //if -> patch, try to download patches, otherwise we fallback to full download
    let download = if let UpdateType::Patch(version_diffs) = &update {
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
        let progress_value = log_progress.get_progress();
        let _ = tx.send(MainMessage::CreateLog(log_progress)).await;

        let increment = (100 / version_diffs.len() as u64) as u8;

        let tasks = version_diffs.iter().map(|diff| {
            let api = api.clone();
            let progress = progress_value.clone();
            let data = data.clone();

            async move {
                let res = match api.get_patch_download(diff, &data).await {
                    Ok(res) => res,
                    Err(err) => match err {
                        ApiError::ResponseCode(_) => return Err(PatchError::PatchNotFound),
                        _ => return Err(PatchError::DownloadFailed),
                    },
                };

                let mut temp_file = TempFile::new()
                    .await
                    .map_err(|_| PatchError::DownloadFailed)?;
                let mut bytes = res.bytes_stream();

                while let Some(chunk) = bytes.next().await {
                    match chunk {
                        Err(_) => {
                            return Err(PatchError::DownloadFailed);
                        }
                        Ok(bytes) => {
                            if (temp_file.write_all(&bytes).await).is_err() {
                                return Err(PatchError::DownloadFailed);
                            }
                        }
                    }
                }

                progress.fetch_add(increment, Ordering::Relaxed);

                Ok(temp_file)
            }
        });

        match try_join_all(tasks).await {
            Err(err) => match err {
                PatchError::PatchNotFound => true,
                _ => {
                    state.lock().unwrap().error();
                    progress_state.lock().unwrap().error();
                    main_state.lock().unwrap().error();

                    let _ = tx
                        .send(MainMessage::VersionUpdateFailed(
                            PopupBuilder::default()
                                .title("Updating failed")
                                .content(match err {
                                    PatchError::PatchNotFound => {
                                        panic!("This case should be unreachable")
                                    }
                                    PatchError::DownloadFailed => {
                                        "Failed to download patch.".to_string()
                                    }
                                })
                                .build()
                                .unwrap(),
                        ))
                        .await;
                    return;
                }
            },
            Ok(files) => {
                let _ = progress_value
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |_| Some(100));
                progress_state.lock().unwrap().finish();
                state.lock().unwrap().finish();

                let binary = match find_factorio_binary(&data.path).await {
                    Some(binary) => binary,
                    None => {
                        let _ = tx
                            .send(MainMessage::VersionUpdateFailed(
                                PopupBuilder::default()
                                    .title("Updating failed")
                                    .content("Failed to locate Factorio binary for applying patch.")
                                    .build()
                                    .unwrap(),
                            ))
                            .await;
                        main_state.lock().unwrap().error();
                        return;
                    }
                };

                let log = LogBuilder::text("Applying patches...")
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
                let progress_value = log_progress.get_progress();
                let _ = tx.send(MainMessage::CreateLog(log_progress)).await;

                let increment = (100 / files.len() as u64) as u8;

                for file in files {
                    //spawn subcommand
                    match Command::new(&binary)
                        .arg("--apply-update")
                        .arg(file.file_path())
                        .kill_on_drop(true)
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status()
                        .await
                    {
                        Ok(status) if status.success() => {}
                        _ => {
                            state.lock().unwrap().error();
                            progress_state.lock().unwrap().error();
                            main_state.lock().unwrap().error();

                            let _ = tx
                                .send(MainMessage::VersionUpdateFailed(
                                    PopupBuilder::default()
                                        .title("Updating failed")
                                        .content("Failed to apply patch.")
                                        .build()
                                        .unwrap(),
                                ))
                                .await;
                            return;
                        }
                    };

                    progress_value.fetch_add(increment, Ordering::Relaxed);
                }

                let _ = progress_value
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |_| Some(100));
                progress_state.lock().unwrap().finish();
                state.lock().unwrap().finish();

                false
            }
        }
    } else {
        true
    };

    if download {
        let res = match get_download_link(
            tx.clone(),
            &api,
            &data.version,
            &data.platform,
            &Item::Stable(Stable {
                stable: target_version.clone(),
            }),
        )
        .await
        {
            Some(res) => res,
            None => {
                main_state.lock().unwrap().error();
                return;
            }
        };

        let file = match download_archive(tx.clone(), res).await {
            Some(file) => file,
            None => {
                main_state.lock().unwrap().error();
                return;
            }
        };

        //remove old installation
        let to_remove = &["data", "bin", "doc-html", "tests"];

        join_all(
            to_remove
                .iter()
                .map(|dir| {
                    let path = data.path.join(dir);
                    async move {
                        if path.exists() {
                            let _ = tokio::fs::remove_dir_all(path).await;
                        }
                    }
                })
                .collect::<Vec<_>>(),
        )
        .await;

        if let None = extract(tx.clone(), file, &data.platform, &data.path).await {
            main_state.lock().unwrap().error();
            return;
        }
    }

    main_state.lock().unwrap().finish();
    let _ = tx
        .send(MainMessage::VersionUpdated(uuid, target_version))
        .await;
}

pub async fn delete_version(tx: Sender<MainMessage>, uuid: uuid::Uuid, data: InstalledVersion) {
    let main_log = LogBuilder::text(format!(
        "Deleting Factorio{} {} v{}...",
        match &data.version {
            Version::Vanilla => "",
            Version::SpaceAge => " Space Age",
            Version::Headless => " Headless",
        },
        data.platform,
        data.current_version,
    ))
    .state(LogState::default())
    .build()
    .unwrap();
    let main_state = main_log.state.clone();
    let _ = tx.send(MainMessage::CreateLog(main_log)).await;

    //remove installation
    if data.path.exists() {
        match tokio::fs::remove_dir_all(&data.path).await {
            Ok(_) => {}
            Err(_) => {
                main_state.lock().unwrap().error();
                return;
            }
        }
    }

    main_state.lock().unwrap().finish();

    let _ = tx.send(MainMessage::VersionDeleted(uuid)).await;
}
