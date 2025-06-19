use serde::{Deserialize, Serialize};

use crate::{ej_client::api::EjClientPost, ej_job::api::EjJob};

#[derive(Debug, Serialize, Deserialize)]
pub enum EjServerMessage {
    Run(EjJob),
    Close,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EjClientMessage {
    Results { results: serde_json::Value },
    JobLog { log: String },
    JobFailure,
    JobSucess,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EjSocketMessage {
    CreateRootUser(EjClientPost),
    Dispatch(EjJob),
}
