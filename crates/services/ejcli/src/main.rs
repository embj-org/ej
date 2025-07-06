//! EJ Command Line Interface (ejcli)
//!
//! A testing and setup tool for the EJ system. Primarily used for:
//!
//! - **Initial Setup**: Create the first user and builder when setting up EJD
//! - **System Testing**: Dispatch test jobs to verify the infrastructure works correctly
//! - **Debug Output**: Print job results and logs directly to the screen for debugging
//!
//! ejcli is designed for system administrators and developers to bootstrap
//! and test the EJ infrastructure. It communicates with EJD via Unix domain sockets
//! for local operations and HTTP for remote builder setup.

mod cli;
mod commands;

use clap::Parser;
use cli::{Cli, Commands};
use commands::{handle_create_builder, handle_create_root_user, handle_dispatch};
use ej_dispatcher_sdk::{ejjob::EjJobType, prelude::*};

use crate::commands::{handle_fetch_jobs, handle_fetch_run_results};

/// Main entry point for the EJ CLI testing and setup tool.
///
/// Parses command line arguments and dispatches to the appropriate handler
/// for system setup, testing, and debugging operations.
///
/// # Examples
///
/// ```bash
/// # Initial setup: Create the first root user
/// ejcli create-root-user --socket /tmp/ejd.sock --name admin --secret password
///
/// # Setup: Create a builder for job execution
/// ejcli create-builder --server http://dispatcher:8080 --name builder-1 --secret token
///
/// # Testing: Dispatch a test build job and view results
/// ejcli dispatch-build --socket /tmp/ejd.sock --seconds 300 --commit-hash abc123 --remote-url https://github.com/user/repo.git
///
/// # Testing: Dispatch a test run job and view logs
/// ejcli dispatch-run --socket /tmp/ejd.sock --seconds 600 --commit-hash def456 --remote-url https://github.com/user/repo.git
/// ```
#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::DispatchBuild { socket, job } => {
            handle_dispatch(&socket, job, EjJobType::Build).await
        }
        Commands::DispatchRun { socket, job } => {
            handle_dispatch(&socket, job, EjJobType::BuildAndRun).await
        }
        Commands::CreateRootUser { socket, client } => {
            handle_create_root_user(&socket, client).await
        }
        Commands::CreateBuilder { server, client } => handle_create_builder(&server, client).await,
        Commands::FetchJobs {
            socket,
            commit_hash,
        } => handle_fetch_jobs(&socket, commit_hash).await,
        Commands::FetchRunResult { socket, job_id } => {
            handle_fetch_run_results(&socket, job_id).await
        }
    };

    if let Err(ref e) = result {
        log::error!("Error: {}", e);
    }

    result
}
