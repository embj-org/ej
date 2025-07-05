//! Command-line interface for the EJ Builder Service.
//!
//! Defines the CLI structure and commands for ejb.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Command-line interface for the EJ Builder Service.
#[derive(Parser)]
#[command(name = "ejb")]
#[command(about = "EJ Builder - Build and run applications across multiple configurations")]
pub struct Cli {
    /// Path to the configuration file
    #[arg(short, long)]
    pub config: PathBuf,

    /// Builder id (can also be set via EJB_ID environment variable)
    #[arg(short, long)]
    pub id: Option<String>,

    /// Builder authentication token (can also be set via EJB_TOKEN environment variable)
    #[arg(short, long)]
    pub token: Option<String>,

    /// Builder socket used to communicate with child processes
    #[arg(short, long)]
    pub socket_path: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands for the EJ Builder Service.
#[derive(Subcommand)]
pub enum Commands {
    /// Parse and display the configuration file
    Parse,
    /// Parse and run every configuration
    Validate,

    /// Check out source code from a remote repository
    Checkout {
        /// Git commit hash
        #[arg(long)]
        commit_hash: String,

        /// Git remote url
        #[arg(long)]
        remote_url: String,

        /// Optional git remote token
        #[arg(long)]
        remote_token: Option<String>,
    },
    /// Run the builder and connect to the server via websockets
    Connect {
        /// Server URL to connect to
        #[arg(short, long)]
        server: String,
    },
}
