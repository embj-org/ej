use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum EjServerMessage {
    Build,
    Run,
    Close,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum EjClientMessage {
    Results { results: serde_json::Value },
    JobLog { log: String },
    JobFailure,
    JobSucess,
}
