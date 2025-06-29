use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    db::connection::DbConnection, ej_config::ej_board_config::EjBoardConfigApi,
    ej_job::db::EjJobCreate, prelude::*,
};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum EjJobType {
    Build = 0,
    BuildAndRun = 1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EjJob {
    pub job_type: EjJobType,
    pub commit_hash: String,
    pub remote_url: String,
    pub remote_token: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct EjDeployableJob {
    pub id: Uuid,
    pub job_type: EjJobType,
    pub commit_hash: String,
    pub remote_url: String,
    pub remote_token: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EjJobCancelReason {
    NoBuilders,
    Timeout,
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EjJobUpdate {
    JobStarted {
        nb_builders: usize,
    },
    JobCancelled(EjJobCancelReason),
    JobAddedToQueue {
        queue_position: usize,
    },
    JobFinished {
        success: bool,
        logs: Vec<(EjBoardConfigApi, String)>,
    },
    RunFinished {
        success: bool,
        results: Vec<(EjBoardConfigApi, String)>,
    },
}

impl EjJob {
    pub fn create(self, connection: &mut DbConnection) -> Result<EjDeployableJob> {
        let job = EjJobCreate {
            commit_hash: self.commit_hash,
            remote_url: self.remote_url,
            job_type: self.job_type as i32,
        };
        let job = job.save(connection)?;

        Ok(EjDeployableJob {
            id: job.id,
            job_type: job.job_type.into(),
            commit_hash: job.commit_hash,
            remote_url: job.remote_url,
            remote_token: self.remote_token,
        })
    }
}
impl fmt::Display for EjJobType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EjJobType::Build => write!(f, "Build"),
            EjJobType::BuildAndRun => write!(f, "Build and Run"),
        }
    }
}

impl fmt::Display for EjDeployableJob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let token_status = if self.remote_token.is_some() {
            "with token"
        } else {
            "without token"
        };
        write!(
            f,
            "Job {} ({}) - Commit: {} from {} {}",
            self.id, self.job_type, self.commit_hash, self.remote_url, token_status
        )
    }
}

impl fmt::Display for EjJobUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EjJobUpdate::JobStarted { nb_builders } => {
                write!(f, "Job started with {} builder(s)", nb_builders)
            }
            EjJobUpdate::JobCancelled(reason) => {
                write!(f, "Job cancelled: {}", reason)
            }
            EjJobUpdate::JobAddedToQueue { queue_position } => {
                write!(f, "Job added to queue at position {}", queue_position)
            }
            EjJobUpdate::JobFinished { success, logs } => {
                let status = if *success {
                    "successfully"
                } else {
                    "with failures"
                };
                writeln!(f, "\n=======================================")?;
                writeln!(
                    f,
                    "Job finished {} with {} log entries:",
                    status,
                    logs.len()
                )?;
                for (board, log) in logs {
                    writeln!(f, "=======================================")?;
                    writeln!(f, "{}", board)?;
                    writeln!(f, "=======================================")?;
                    writeln!(f, "{}", log)?;
                }
                writeln!(f, "=======================================")?;
                Ok(())
            }
            EjJobUpdate::RunFinished { success, results } => {
                let status = if *success {
                    "successfully"
                } else {
                    "with failures"
                };
                writeln!(f, "\n=======================================")?;
                writeln!(
                    f,
                    "Run finished {} with {} result entries:",
                    status,
                    results.len()
                )?;
                for (board, result) in results {
                    writeln!(f, "=======================================")?;
                    writeln!(f, "{}", board)?;
                    writeln!(f, "=======================================")?;
                    writeln!(f, "{}", result)?;
                }
                writeln!(f, "=======================================")?;
                Ok(())
            }
        }
    }
}
impl fmt::Display for EjJobCancelReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EjJobCancelReason::NoBuilders => write!(f, "no builders"),
            EjJobCancelReason::Timeout => write!(f, "job timed out"),
        }
    }
}
