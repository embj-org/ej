//! Command-line interface definitions for ejcli.
//!
//! Defines the CLI structure, commands, and arguments for the EJ testing
//! and setup tool.

use clap::{Args, Parser, Subcommand};
use std::{path::PathBuf, time::Duration};

/// EJ Command Line Interface for testing and system setup.
#[derive(Parser)]
#[command(name = "ejc")]
#[command(about = "EJ CLI - Testing and setup tool for the EJ system")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands for the EJ CLI testing and setup tool.
#[derive(Subcommand)]
pub enum Commands {
    /// Dispatch a test build job (results printed to screen)
    DispatchBuild {
        /// Path to the EJD's unix socket
        #[arg(short, long)]
        socket: PathBuf,
        #[command(flatten)]
        job: DispatchArgs,
    },

    /// Dispatch a test run job (results printed to screen)
    DispatchRun {
        /// Path to the EJD's unix socket
        #[arg(short, long)]
        socket: PathBuf,
        #[command(flatten)]
        job: DispatchArgs,
    },

    /// Create the initial root user (for system setup)
    CreateRootUser {
        /// Path to the EJD's unix socket
        #[arg(short, long)]
        socket: PathBuf,

        #[command(flatten)]
        client: UserArgs,
    },

    /// Create a new builder (for system setup)
    CreateBuilder {
        /// Server url
        #[arg(short, long)]
        server: String,

        #[command(flatten)]
        client: UserArgs,
    },
}

/// Arguments for dispatching a job.
#[derive(Args)]
pub struct DispatchArgs {
    /// The maximum job duration in seconds
    #[arg(long)]
    pub seconds: u64,

    /// Git commit hash
    #[arg(long)]
    pub commit_hash: String,

    /// Git remote url
    #[arg(long)]
    pub remote_url: String,

    /// Optional git remote token
    #[arg(long)]
    pub remote_token: Option<String>,
}
/// User arguments for creating a new user or builder.
#[derive(Args)]
pub struct UserArgs {
    /// User name
    #[arg(long)]
    pub username: String,

    /// User password
    /// Recomended to keep this empty and set it when prompted
    #[arg(long)]
    pub password: Option<String>,
}
