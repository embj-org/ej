use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    ej_client::api::{EjClientApi, EjClientPost},
    ej_job::api::{EjDeployableJob, EjJob, EjJobUpdate},
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EjServerMessage {
    Build(EjDeployableJob),
    BuildAndRun(EjDeployableJob),
    Close,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EjClientMessage {}

#[derive(Debug, Serialize, Deserialize)]
pub enum EjSocketClientMessage {
    CreateRootUser(EjClientPost),
    Dispatch(EjJob),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EjSocketServerMessage {
    CreateRootUserOk(EjClientApi),
    CreateRootUserError,
    DispatchOk(EjDeployableJob),
    JobUpdate(EjJobUpdate),
    Error(String),
}

impl fmt::Display for EjSocketServerMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EjSocketServerMessage::CreateRootUserOk(ej_client_api) => {
                write!(f, "Root user created successfully: {}", ej_client_api)
            }
            EjSocketServerMessage::CreateRootUserError => {
                write!(f, "Failed to create root user")
            }
            EjSocketServerMessage::DispatchOk(ej_deployable_job) => {
                write!(f, "Job dispatched successfully: {}", ej_deployable_job)
            }
            EjSocketServerMessage::JobUpdate(ej_job_update) => {
                write!(f, "Job update: {}", ej_job_update)
            }
            EjSocketServerMessage::Error(error_msg) => {
                write!(f, "Error: {}", error_msg)
            }
        }
    }
}
