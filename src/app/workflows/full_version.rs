use ratatui::text::Line;
use tokio::sync::mpsc::Sender;

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

pub enum InstallationErrors {}

pub async fn install_full_version(
    tx: Sender<MainMessage>,
    api: Api,
    path: String,
    platform: Platform,
    version: Version,
    patch: Item,
) {
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

    let res = match res {
        Ok(res) => res,
        Err(popup) => {
            let _ = tx
                .send(MainMessage::OpenPopup((OpenedPopup::ErrorNotify, popup)))
                .await;
            state.lock().unwrap().error();
            return;
        }
    };

    let message_log = LogBuilder::text("Downloading archive ");
}
