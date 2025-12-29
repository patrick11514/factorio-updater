use std::sync::{Arc, Mutex};

use crate::app::{
    api::{ApiError, Response, structs::Updates},
    components::log::{Log, LogState},
    screens::main::components::run::{InstalledVersionDetails, RunState},
};

type State = Arc<Mutex<LogState>>;

pub enum MainMessage {
    CreateLog(Log),
    CheckLogin(Result<Option<()>, ApiError>, State),
    LoadVersions(Result<Response<Updates>, ApiError>, State),
    ChangeRunState(RunState),
    VersionDetails(Vec<InstalledVersionDetails>, State),
}
