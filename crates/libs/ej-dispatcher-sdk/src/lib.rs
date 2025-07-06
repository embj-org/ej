//! Dispatcher SDK for the EJ framework.
//!
//! Provides client interfaces for communicating with the EJ dispatcher service.
//!
//! # Usage
//!
//! ```rust,no_run
//! use ej_dispatcher_sdk::{EjJob, EjJobType, dispatch_run};
//! use std::time::Duration;
//! use std::path::Path;
//!
//! # tokio_test::block_on(async {
//!
//! // Dispatch a run job to the dispatcher and wait for completion
//! let result = dispatch_run(
//!     Path::new("/tmp/ejd.sock"),
//!     "abc123".to_string(),
//!     "https://github.com/user/repo.git".to_string(),
//!     None,
//!     Duration::from_secs(600),
//! ).await.unwrap();
//!# });
//! ```

use crate::{ejsocket_message::EjSocketClientMessage, prelude::*};
use std::{collections::HashMap, fmt, path::Path, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    net::UnixStream,
};
use tracing::{error, info};
use uuid::Uuid;

pub use crate::{
    build::dispatch_build,
    ejjob::{
        EjBuildResult, EjDeployableJob, EjJob, EjJobCancelReason, EjJobType, EjJobUpdate,
        EjRunResult,
    },
    fetch_jobs::fetch_jobs,
    fetch_run_result::fetch_run_result,
    run::dispatch_run,
};

pub mod build;
pub mod ejbuilder;
pub mod ejclient;
pub mod ejjob;
pub mod ejsocket_message;
pub mod ejws_message;
pub mod error;
pub mod fetch_jobs;
pub mod fetch_run_result;
pub mod prelude;
pub mod run;
mod socket;

/// Dispatch a job to the EJ dispatcher.
///
/// Sends a job request to the dispatcher via Unix socket with a maximum duration timeout.
///
/// # Arguments
///
/// * `stream` - Unix socket connection to the dispatcher
/// * `job` - Job configuration to dispatch
/// * `max_duration` - Maximum time to wait for job completion
/// ```
async fn dispatch(stream: &mut UnixStream, job: EjJob, max_duration: Duration) -> Result<()> {
    let message = EjSocketClientMessage::Dispatch {
        job,
        timeout: max_duration,
    };

    let payload = serde_json::to_string(&message)?;
    stream.write_all(payload.as_bytes()).await;
    stream.write_all(b"\n").await;
    stream.flush().await;
    Ok(())
}
