use std::sync::{Arc, Mutex};

use crate::app::{
    api::{ApiError, Response, structs::Updates},
    components::log::{Log, LogState},
};

type State = Arc<Mutex<LogState>>;

pub enum MainMessage {
    CreateLog(Log),
    CheckLogin(Result<Option<()>, ApiError>, State),
    LoadVersions(Result<Response<Updates>, ApiError>, State),
}
