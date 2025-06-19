use ej::prelude::*;

mod cli;
mod commands;
mod models;

use clap::Parser;
use cli::{Cli, Commands};
use commands::{handle_create_builder, handle_create_root_user, handle_dispatch};
use ej::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Dispatch { socket, job } => handle_dispatch(&socket, job).await,
        Commands::CreateRootUser { socket, client } => {
            handle_create_root_user(&socket, client).await
        }
        Commands::CreateBuilder { server, client } => handle_create_builder(&server, client).await,
    };

    if let Err(ref e) = result {
        log::error!("Error: {}", e);
    }

    result
}
