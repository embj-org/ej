use serde::{Deserialize, Serialize};

use crate::{
    ej_client::api::{EjClientApi, EjClientPost},
    ej_job::api::{EjDeployableJob, EjJob, EjJobUpdate},
};

#[derive(Debug, Serialize, Deserialize)]
pub enum EjServerMessage {
    Build(EjDeployableJob),
    Run(EjDeployableJob),
    Close,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EjClientMessage {}

#[derive(Debug, Serialize, Deserialize)]
pub enum EjSocketServerMessage {
    CreateRootUserOk(EjClientApi),
    CreateRootUserError,
    DispatchOk(EjDeployableJob),
    JobUpdate(EjJobUpdate),
    BuildJobResult,
    RunJobResult,
    Error(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EjSocketClientMessage {
    CreateRootUser(EjClientPost),
    Dispatch(EjJob),
}
