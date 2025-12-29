use tokio::sync::mpsc::Sender;

use crate::app::{
    api::{self, Api},
    components::log::{LogBuilder, LogState},
    screens::main::message::MainMessage,
};

pub async fn check_credentials(api: &Api, tx: &Sender<MainMessage>) {
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
            }
            false => {
                tx.send(MainMessage::CheckLogin(Ok(None), state))
                    .await
                    .unwrap();
                return;
            }
        },
        Err(err) => {
            tx.send(MainMessage::CheckLogin(Err(err), state))
                .await
                .unwrap();
            return;
        }
    };
}

pub async fn fetch_versions(api: &Api, tx: &Sender<MainMessage>) {
    let log = LogBuilder::text("Fetching available versions...")
        .state(LogState::default())
        .build()
        .unwrap();

    let state = log.state.clone();
    tx.send(MainMessage::CreateLog(log)).await.unwrap();
    tx.send(MainMessage::LoadVersions(api.get_versions().await, state))
        .await
        .unwrap();
}
