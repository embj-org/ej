mod build;
mod checkout;
mod cli;
mod commands;
mod connection;
mod logs;
mod run;

use clap::Parser;
use cli::{Cli, Commands};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    checkout::handle_checkout,
    commands::{handle_parse, handle_run_and_build},
    connection::handle_connect,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ejb=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Parse => handle_parse(&cli.config),
        Commands::Checkout {
            commit_hash,
            remote_url,
            remote_token,
        } => handle_checkout(&cli.config, commit_hash, remote_url, remote_token),
        Commands::Validate => handle_run_and_build(&cli.config),
        Commands::Connect { server } => {
            handle_connect(&cli.config, &server, cli.id, cli.token).await
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
