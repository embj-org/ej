use clap::{Args, Parser, Subcommand};
use ej::{
    ej_client::api::EjClientPost,
    ej_job::api::{EjJob, EjJobType},
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ejc")]
#[command(about = "EJ  - Build and run applications across multiple configurations")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Dispatch a new build job
    DispatchBuild {
        /// Path to the EJD's unix socket
        #[arg(short, long)]
        socket: PathBuf,
        #[command(flatten)]
        job: DispatchArgs,
    },

    /// Dispatch a new run job
    DispatchRun {
        /// Path to the EJD's unix socket
        #[arg(short, long)]
        socket: PathBuf,
        #[command(flatten)]
        job: DispatchArgs,
    },

    /// Create a new user
    CreateRootUser {
        /// Path to the EJD's unix socket
        #[arg(short, long)]
        socket: PathBuf,

        #[command(flatten)]
        client: UserArgs,
    },

    /// Create a new builder
    CreateBuilder {
        /// Server url
        #[arg(short, long)]
        server: String,

        #[command(flatten)]
        client: UserArgs,
    },
}
#[derive(Args)]
pub struct DispatchArgs {
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
