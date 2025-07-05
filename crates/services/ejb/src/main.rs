//! EJ Builder Service (ejb)
//!
//! A builder service that handles code compilation, validation, and execution
//! for the EJ job execution system. The builder can run in different modes:
//!
//! - **Parse**: Parse and validate build configurations
//! - **Checkout**: Check out source code from remote repositories  
//! - **Validate**: Run build and validation processes
//! - **Connect**: Connect to the EJD dispatcher service for job execution
//!
//! ## Communication Architecture
//!
//! EJB communicates with EJD (dispatcher) using:
//! - **REST API**: For builder registration, login, and configuration upload
//! - **WebSocket**: For real-time job assignment and result reporting
//! - **Unix Sockets**: For local communication with child processes
//!
//! The builder authenticates with EJD using JWT tokens and maintains a persistent
//! WebSocket connection to receive job assignments and report results.

mod build;
mod builder;
mod checkout;
mod cli;
mod commands;
mod common;
mod connection;
mod error;
mod logs;
mod prelude;
mod run;
mod run_output;
use std::path::PathBuf;

use clap::Parser;
use cli::{Cli, Commands};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::prelude::*;
use crate::{
    builder::Builder,
    checkout::handle_checkout,
    commands::{handle_parse, handle_run_and_build},
    connection::handle_connect,
};

/// Main entry point for the EJ Builder Service.
///
/// Initializes logging, parses command line arguments, creates a builder instance,
/// and dispatches to the appropriate command handler based on the CLI arguments.
///
/// # Examples
///
/// ```bash
/// # Parse build configuration
/// ejb parse --config config.toml
///
/// # Check out source code
/// ejb checkout --commit-hash abc123 --remote-url https://github.com/user/repo.git
///
/// # Validate build
/// ejb validate --config config.toml
///
/// # Connect to dispatcher
/// ejb connect --server http://dispatcher:8080 --id builder-123 --token builder_jwt_token
/// ```
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ejb=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();
    let default_socket_path = PathBuf::from("/tmp/ejb.sock");
    let builder =
        Builder::create(cli.config, cli.socket_path.unwrap_or(default_socket_path)).await?;

    match cli.command {
        Commands::Parse => handle_parse(&builder),
        Commands::Checkout {
            commit_hash,
            remote_url,
            remote_token,
        } => handle_checkout(&builder, commit_hash, remote_url, remote_token),
        Commands::Validate => handle_run_and_build(&builder),
        Commands::Connect { server } => handle_connect(builder, &server, cli.id, cli.token).await,
    }
}
