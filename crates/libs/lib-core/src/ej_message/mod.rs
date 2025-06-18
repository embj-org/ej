use serde::{Deserialize, Serialize};

use crate::ej_job::api::EjJob;

#[derive(Debug, Serialize)]
pub enum EjServerMessage {
    Run(EjJob),
    Close,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum EjClientMessage {
    Results { results: serde_json::Value },
    JobLog { log: String },
    JobFailure,
    JobSucess,
}
