//! Job management types and utilities.

pub mod results;

use std::fmt;

use ej_config::ej_board_config::EjBoardConfigApi;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of job to execute.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum EjJobType {
    /// Build only (compile/prepare without running).
    Build = 0,
    /// Build and run (compile and execute tests).
    BuildAndRun = 1,
}

impl From<i32> for EjJobType {
    fn from(value: i32) -> Self {
        match value {
            0 => EjJobType::Build,
            1 => EjJobType::BuildAndRun,
            _ => unreachable!(),
        }
    }
}

/// Job configuration for the dispatcher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EjJob {
    /// Type of job to execute.
    pub job_type: EjJobType,
    /// Git commit hash to build/run.
    pub commit_hash: String,
    /// Git repository URL.
    pub remote_url: String,
    /// Optional authentication token for private repositories.
    pub remote_token: Option<String>,
}
impl EjJob {
    pub fn new(
        job_type: EjJobType,
        commit_hash: impl Into<String>,
        remote_url: impl Into<String>,
        remote_token: Option<String>,
    ) -> Self {
        Self {
            job_type,
            commit_hash: commit_hash.into(),
            remote_url: remote_url.into(),
            remote_token,
        }
    }
}

/// Deployable job with assigned ID.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct EjDeployableJob {
    /// Unique job identifier.
    pub id: Uuid,
    /// Type of job to execute.
    pub job_type: EjJobType,
    /// Git commit hash to build/run.
    pub commit_hash: String,
    /// Git repository URL.
    pub remote_url: String,
    /// Optional authentication token for private repositories.
    pub remote_token: Option<String>,
}

/// Reason for job cancellation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EjJobCancelReason {
    /// No builders available to execute the job.
    NoBuilders,
    /// Job exceeded maximum execution time.
    Timeout,
}

/// Job status updates from the dispatcher.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EjJobUpdate {
    /// Job has started execution.
    JobStarted {
        /// Number of builders assigned to the job.
        nb_builders: usize,
    },
    /// Job was cancelled.
    JobCancelled(EjJobCancelReason),
    /// Job was added to the execution queue.
    JobAddedToQueue {
        /// Position in the queue.
        queue_position: usize,
    },
    /// Build phase completed.
    BuildFinished(EjBuildResult),
    /// Run phase completed.
    RunFinished(EjRunResult),
}

/// Build operation result.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjBuildResult {
    /// Build logs per board configuration.
    pub logs: Vec<(EjBoardConfigApi, String)>,
    /// Whether the build was successful.
    pub success: bool,
}

/// Run operation result.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjRunResult {
    /// Run logs per board configuration.
    pub logs: Vec<(EjBoardConfigApi, String)>,
    /// Run results per board configuration.
    pub results: Vec<(EjBoardConfigApi, String)>,
    /// Whether the run was successful.
    pub success: bool,
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
            EjJobUpdate::BuildFinished(result) => {
                write!(f, "{}", result)
            }
            EjJobUpdate::RunFinished(result) => {
                write!(f, "{}", result)
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
impl fmt::Display for EjBuildResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.success {
            "successfully"
        } else {
            "with failures"
        };
        writeln!(f, "\n=======================================")?;
        writeln!(
            f,
            "Build finished {} with {} log entries:",
            status,
            self.logs.len()
        )?;
        for (board, log) in self.logs.iter() {
            writeln!(f, "=======================================")?;
            writeln!(f, "{}", board)?;
            writeln!(f, "=======================================")?;
            writeln!(f, "{}", log)?;
        }
        writeln!(f, "=======================================")
    }
}

impl fmt::Display for EjRunResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.success {
            "successfully"
        } else {
            "with failures"
        };
        writeln!(f, "\n=======================================")?;
        writeln!(
            f,
            "Run finished {} with {} log entries:",
            status,
            self.logs.len()
        )?;
        for (board, log) in self.logs.iter() {
            writeln!(f, "=======================================")?;
            writeln!(f, "{}", board)?;
            writeln!(f, "=======================================")?;
            writeln!(f, "{}", log)?;
        }
        writeln!(f, "=======================================")?;
        writeln!(f, "\n=======================================")?;
        writeln!(
            f,
            "Run finished {} with {} result entries:",
            status,
            self.results.len()
        )?;
        for (board, result) in self.results.iter() {
            writeln!(f, "=======================================")?;
            writeln!(f, "{}", board)?;
            writeln!(f, "=======================================")?;
            writeln!(f, "{}", result)?;
        }
        writeln!(f, "=======================================")
    }
}
