use ej::prelude::*;

mod cli;
mod commands;

use clap::Parser;
use cli::{Cli, Commands};
use commands::handle_dispatch;
use ej::prelude::*;

use crate::commands::handle_create_user;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Dispatch { job } => handle_dispatch(&cli.socket, job).await,
        Commands::CreateUser { client } => handle_create_user(&cli.socket, client).await,
    };

    if let Err(ref e) = result {
        log::error!("Error: {}", e);
    }

    result
}
