use clap::{Parser, Subcommand};
use std::path::PathBuf;

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

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Parse and display the configuration file
    Parse,
    /// Parse and run every configuration
    Validate,
    /// Run the builder and connect to the server via websockets
    Run {
        /// Server URL to connect to
        #[arg(short, long)]
        server: String,
    },
}
