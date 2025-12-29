use std::sync::{Arc, Mutex};

use crate::app::{
    api::ApiError,
    components::log::{Log, LogState},
};

pub enum MainMessage {
    CreateLog(Log),
    CheckLogin(Result<Option<()>, ApiError>, Arc<Mutex<LogState>>),
}
