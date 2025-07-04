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
