//! Unix socket message types for dispatcher communication.

use std::{fmt, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{
    ejclient::{EjClientApi, EjClientPost},
    ejjob::{EjDeployableJob, EjJob, EjJobUpdate},
};

/// Messages sent from client to dispatcher via Unix socket.
#[derive(Debug, Serialize, Deserialize)]
pub enum EjSocketClientMessage {
    /// Create root user request.
    CreateRootUser(EjClientPost),
    /// Dispatch job request.
    Dispatch {
        /// Job configuration.
        job: EjJob,
        /// Maximum execution timeout.
        timeout: Duration,
    },
}

/// Messages sent from dispatcher to client via Unix socket.
#[derive(Debug, Serialize, Deserialize)]
pub enum EjSocketServerMessage {
    /// Root user creation successful.
    CreateRootUserOk(EjClientApi),
    /// Root user creation failed.
    CreateRootUserError,
    /// Job dispatch successful.
    DispatchOk(EjDeployableJob),
    /// Job status update.
    JobUpdate(EjJobUpdate),
    /// General error message.
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
