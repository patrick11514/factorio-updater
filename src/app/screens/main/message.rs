use std::sync::{Arc, Mutex};

use crate::app::{api::ApiError, components::log::LogState};

pub enum MainMessage {
    CheckLogin(Result<Option<()>, ApiError>, Arc<Mutex<LogState>>),
}
