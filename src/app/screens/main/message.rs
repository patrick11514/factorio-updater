use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::app::{
    api::{ApiError, Response, structs::Updates},
    components::{
        log::{Log, LogState},
        popup::Popup,
    },
    config::InstalledVersion,
    screens::main::components::{
        run::{InstalledVersionDetails, RunState},
        tick::OpenedPopup,
    },
};

type State = Arc<Mutex<LogState>>;

pub enum MainMessage {
    CreateLog(Log),
    CheckLogin(Result<Option<()>, ApiError>, State),
    LoadVersions(Result<Response<Updates>, ApiError>, State),
    ChangeRunState(RunState),
    VersionDetails(HashMap<uuid::Uuid, InstalledVersionDetails>, State),
    OpenPopup(OpenedPopup, Popup<'static>),
    VersionInstalled(InstalledVersion),
    VersionUpdateFailed(Popup<'static>),
    VersionUpdated(uuid::Uuid, String),
}
