use futures_util::StreamExt;
use tar::Archive;
use tokio::{io::AsyncWriteExt, sync::mpsc::Sender};
use xz2::bufread::XzDecoder;
use zip::read::root_dir_common_filter;

use std::{
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
    sync::atomic::Ordering,
};

use crate::app::{
    api::{
        Api,
        structs::{Item, Platform, Version},
    },
    components::{
        log::{LogBuilder, LogState},
        popup::PopupBuilder,
    },
    screens::main::{components::tick::OpenedPopup, message::MainMessage},
};
use async_tempfile::TempFile;

pub async fn get_download_link(
    tx: Sender<MainMessage>,
    api: &Api,
    version: &Version,
    platform: &Platform,
    patch: &Item,
) -> Option<reqwest::Response> {
    let log = LogBuilder::text("Getting download link...")
        .state(LogState::default())
        .build()
        .unwrap();
    let state = log.state.clone();
    let _ = tx.send(MainMessage::CreateLog(log)).await;

    let res = match api
        .get_stable_download_link((version, platform).into(), &patch)
        .await
    {
        Ok(res) => match res {
            Some(res) => Ok(res),
            None => Err(PopupBuilder::default()
                .error()
                .content("This version is not available for download.")
                .title("Download Unavailable")
                .build()
                .unwrap()),
        },
        Err(err) => Err(PopupBuilder::default()
            .error()
            .content(format!("Failed to get download link: {}", err))
            .title("Download Link Error")
            .build()
            .unwrap()),
    };

    match res {
        Ok(res) => {
            state.lock().unwrap().finish();
            Some(res)
        }
        Err(popup) => {
            let _ = tx
                .send(MainMessage::OpenPopup((OpenedPopup::ErrorNotify, popup)))
                .await;
            state.lock().unwrap().error();
            return None;
        }
    }
}

pub async fn download_archive(tx: Sender<MainMessage>, res: reqwest::Response) -> Option<TempFile> {
    let log = LogBuilder::text("Downloading archive...")
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

    let mut temp_file = match TempFile::new().await {
        Ok(file) => file,
        Err(_) => {
            let popup = PopupBuilder::default()
                .error()
                .content("Failed to create temporary file for download.")
                .title("File Error")
                .build()
                .unwrap();

            let _ = tx
                .send(MainMessage::OpenPopup((OpenedPopup::ErrorNotify, popup)))
                .await;

            state.lock().unwrap().error();
            progress_state.lock().unwrap().error();
            return None;
        }
    };

    let len = res.content_length().unwrap_or(0);
    let mut stream = res.bytes_stream();
    let mut len_acc = 0u64;

    while let Some(data) = stream.next().await {
        match data {
            Err(err) => {
                let popup = PopupBuilder::default()
                    .error()
                    .content(format!("Download failed: {}", err))
                    .title("Download Error")
                    .build()
                    .unwrap();

                let _ = tx
                    .send(MainMessage::OpenPopup((OpenedPopup::ErrorNotify, popup)))
                    .await;

                state.lock().unwrap().error();
                progress_state.lock().unwrap().error();
                return None;
            }
            Ok(bytes) => {
                len_acc += bytes.len() as u64;

                if let Err(err) = temp_file.write_all(&bytes).await {
                    let popup = PopupBuilder::default()
                        .error()
                        .content(format!("Failed to write to temporary file: {}", err))
                        .title("File Write Error")
                        .build()
                        .unwrap();

                    let _ = tx
                        .send(MainMessage::OpenPopup((OpenedPopup::ErrorNotify, popup)))
                        .await;

                    state.lock().unwrap().error();
                    progress_state.lock().unwrap().error();
                    return None;
                }
                let new_progress = if len > 0 {
                    ((len_acc as f64 / len as f64) * 100.0) as u8
                } else {
                    0
                };
                progress_value.store(new_progress, Ordering::Relaxed);
            }
        }
    }

    state.lock().unwrap().finish();
    progress_state.lock().unwrap().finish();

    Some(temp_file)
}

pub async fn extract(
    tx: Sender<MainMessage>,
    file: TempFile,
    platform: &Platform,
    path: &str,
) -> Option<()> {
    let path = Path::new(path);

    if !path.exists() {
        match fs::create_dir_all(path) {
            Ok(_) => {}
            Err(_) => {
                let popup = PopupBuilder::default()
                    .error()
                    .content(format!("Failed to create folder: {}", path.display()))
                    .title("Extraction Error")
                    .build()
                    .unwrap();

                let _ = tx
                    .send(MainMessage::OpenPopup((OpenedPopup::ErrorNotify, popup)))
                    .await;

                return None;
            }
        }
    }

    let path = path.to_path_buf();
    let platform = platform.clone();

    let log = LogBuilder::text("Extracting archive...")
        .state(LogState::default())
        .build()
        .unwrap();
    let state = log.state.clone();
    let _ = tx.send(MainMessage::CreateLog(log)).await;

    let result = tokio::task::spawn_blocking(move || match platform {
        Platform::Linux32 | Platform::Linux64 => extract_tar_xz(file, path),
        Platform::Win32 | Platform::Win64 => extract_zip(file, path),
        _ => panic!("Unsupported platform for extraction"),
    })
    .await
    .unwrap();

    match result {
        Err(_) => {
            let popup = PopupBuilder::default()
                .error()
                .content("Failed to extract the archive.")
                .title("Extraction Error")
                .build()
                .unwrap();

            let _ = tx
                .send(MainMessage::OpenPopup((OpenedPopup::ErrorNotify, popup)))
                .await;

            state.lock().unwrap().error();
            None
        }
        Ok(_) => {
            state.lock().unwrap().finish();
            Some(())
        }
    }
}

enum ExtractResult {
    FailedOpenFile,
    FailedRead,
    FailedExtract,
}

fn extract_tar_xz(archive: TempFile, target: PathBuf) -> Result<(), ExtractResult> {
    let file = File::open(archive.file_path()).map_err(|_| ExtractResult::FailedOpenFile)?;
    let buf = BufReader::new(file);
    let decoder = XzDecoder::new(buf);
    let mut archive = Archive::new(decoder);

    for file in archive.entries().map_err(|_| ExtractResult::FailedRead)? {
        let mut file = file.map_err(|_| ExtractResult::FailedExtract)?;
        let path = match file.path() {
            Ok(p) => p.into_owned(),
            Err(_) => return Err(ExtractResult::FailedExtract),
        };

        let stripped_path: PathBuf = path.components().skip(1).collect();

        let full_path = target.join(stripped_path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).map_err(|_| ExtractResult::FailedExtract)?;
        }
        file.unpack(&full_path)
            .map_err(|_| ExtractResult::FailedExtract)?;
    }

    Ok(())
}

fn extract_zip(archive: TempFile, target: PathBuf) -> Result<(), ExtractResult> {
    let file = File::open(archive.file_path()).map_err(|_| ExtractResult::FailedOpenFile)?;
    let mut zip = zip::ZipArchive::new(file).map_err(|_| ExtractResult::FailedRead)?;

    zip.extract_unwrapped_root_dir(target, root_dir_common_filter)
        .map_err(|_| ExtractResult::FailedExtract)?;

    Ok(())
}

//pub async fn fetch_all_version
