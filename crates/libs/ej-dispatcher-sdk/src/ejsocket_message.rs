//! Unix socket message types for dispatcher communication.

use std::{fmt, time::Duration};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    EjRunResult,
    ejclient::{EjClientApi, EjClientPost},
    ejjob::{EjDeployableJob, EjJob, EjJobApi, EjJobUpdate},
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
    /// Fetch jobs associated to a commit hash
    FetchJobs { commit_hash: String },

    /// Fetch job results associated to this id
    FetchJobResults { job_id: Uuid },
}

/// Messages sent from dispatcher to client via Unix socket.
#[derive(Debug, Serialize, Deserialize)]
pub enum EjSocketServerMessage {
    /// Root user creation successful.
    CreateRootUserOk(EjClientApi),
    /// Job dispatch successful.
    DispatchOk(EjDeployableJob),
    /// Job status update.
    JobUpdate(EjJobUpdate),
    /// A list of jobs. Response of `EjSocketClientMessage::FetchJobs`
    Jobs(Vec<EjJobApi>),
    /// A run result. Response of `EjSocketClientMessage::FetchJobResults`
    RunResult(EjRunResult),
    /// General error message.
    Error(String),
}

impl fmt::Display for EjSocketServerMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EjSocketServerMessage::CreateRootUserOk(ej_client_api) => {
                write!(f, "Root user created successfully: {}", ej_client_api)
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
            EjSocketServerMessage::Jobs(jobs) => {
                writeln!(f, "== Jobs ==")?;
                for job in jobs {
                    writeln!(f, "{}", job)?;
                }
                writeln!(f, "== Jobs ==")?;
                Ok(())
            }
            EjSocketServerMessage::RunResult(run_result) => write!(f, "{}", run_result),
        }
    }
}
