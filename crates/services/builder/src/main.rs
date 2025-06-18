mod cli;
mod commands;

use clap::Parser;
use cli::{Cli, Commands};
use commands::{handle_parse, handle_run, handle_validate};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
        Commands::Validate => handle_validate(&cli.config),
        Commands::Run { server } => handle_run(&cli.config, &server, cli.token).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
