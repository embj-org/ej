use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    ej_client::api::EjClientPost,
    ej_job::api::{EjDeployableJob, EjJob},
};

#[derive(Debug, Serialize, Deserialize)]
pub enum EjServerMessage {
    Build(EjDeployableJob),
    Run(EjDeployableJob),
    Error(String),
    Close,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EjClientMessage {
    Results {
        job_id: Uuid,
        config_id: Uuid,
        results: String,
    },
    JobLog {
        job_id: Uuid,
        config_id: Uuid,
        log: String,
    },
    BuildSuccess {
        job_id: Uuid,
        builder_id: Uuid,
    },
    BuildFailure {
        job_id: Uuid,
        builder_id: Uuid,
        error: String,
    },
    RunSuccess {
        job_id: Uuid,
        builder_id: Uuid,
    },
    RunFailure {
        job_id: Uuid,
        builder_id: Uuid,
        error: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EjSocketMessage {
    CreateRootUser(EjClientPost),
    Dispatch(EjJob),
}
