use ej::{ej_job::api::EjJobType, prelude::*};

mod cli;
mod commands;

use clap::Parser;
use cli::{Cli, Commands};
use commands::{handle_create_builder, handle_create_root_user, handle_dispatch};
use ej::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::DispatchBuild { socket, job } => {
            handle_dispatch(&socket, job, EjJobType::Build).await
        }

        Commands::DispatchRun { socket, job } => {
            handle_dispatch(&socket, job, EjJobType::Run).await
        }
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
