use clap::{Args, Parser, Subcommand};
use ej::{ej_client::api::EjClientPost, ej_job::api::EjJob};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ejc")]
#[command(about = "EJ  - Build and run applications across multiple configurations")]
pub struct Cli {
    /// Path to the EJD's unix socket
    #[arg(short, long)]
    pub socket: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Dispatch a new job
    Dispatch {
        #[command(flatten)]
        job: DispatchArgs,
    },

    /// Create a new user
    CreateUser {
        #[command(flatten)]
        client: CreateUserArgs,
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

impl From<DispatchArgs> for EjJob {
    fn from(value: DispatchArgs) -> Self {
        Self {
            remote_token: value.remote_token,
            commit_hash: value.commit_hash,
            remote_url: value.remote_url,
        }
    }
}

#[derive(Args)]
pub struct CreateUserArgs {
    /// User name
    #[arg(long)]
    pub name: String,

    /// User password
    #[arg(long)]
    pub password: String,
}

impl From<CreateUserArgs> for EjClientPost {
    fn from(value: CreateUserArgs) -> Self {
        Self {
            name: value.name,
            secret: value.password,
        }
    }
}
