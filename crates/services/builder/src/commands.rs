use ej::ej_config::ej_board::EjBoard;
use ej::ej_config::ej_config::EjConfig;
use ej::prelude::*;
use lib_io::runner::Runner;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use tracing::info;

pub fn handle_parse(config_path: &PathBuf) -> Result<()> {
    info!("Parsing configuration file: {:?}", config_path);

    let config = EjConfig::from_file(config_path)?;

    info!("Configuration parsed successfully");
    info!("Global version: {}", config.global.version);
    info!("Number of boards: {}", config.boards.len());

    for (board_idx, board) in config.boards.iter().enumerate() {
        info!("\nBoard {}: {}", board_idx + 1, board.name);
        info!("  Description: {}", board.description);
        info!("  Configurations: {}", board.configs.len());

        for (config_idx, board_config) in board.configs.iter().enumerate() {
            info!(
                "    Config {}: {}",
                config_idx + 1,
                board_config.description
            );
            info!("      Tags: {:?}", board_config.tags);
            info!("      Build script: {:?}", board_config.build_script);
            info!("      Run script: {:?}", board_config.run_script);
            info!("      Results path: {:?}", board_config.results_path);
            info!("      Library path: {:?}", board_config.library_path);
        }
    }

    Ok(())
}

pub fn run_all_configs(board: EjBoard) {
    for board_config in board.configs.iter() {
        let (tx, rx) = mpsc::channel();
        let should_stop = Arc::new(AtomicBool::new(false));
        let runner = Runner::new(board_config.run_script.clone(), Vec::new());
        let result = thread::spawn(move || runner.run(tx, should_stop));

        while let Ok(event) = rx.recv() {
            match event {
                lib_io::runner::RunEvent::ProcessNewOutputLine(line) => print!("{}", line),
                _ => info!("{:?}", event),
            }
        }
        let result = result.join();
        info!("Result {:?}", result);

        let results = std::fs::read_to_string(board_config.results_path.clone()).unwrap();
        println!("{results}");
    }
}

pub fn handle_validate(config_path: &PathBuf) -> Result<()> {
    info!("Validating configuration file: {:?}", config_path);

    let config = EjConfig::from_file(config_path)?;

    let board_count = config.boards.len();
    for (board_idx, board) in config.boards.iter().enumerate() {
        info!("Board {}/{}: {}", board_idx + 1, board_count, board.name);
        for (config_idx, board_config) in board.configs.iter().enumerate() {
            {
                let (tx, rx) = mpsc::channel();
                let should_stop = Arc::new(AtomicBool::new(false));
                info!("\tConfig {}: {}", config_idx + 1, board_config.description);
                let runner = Runner::new(board_config.build_script.clone(), Vec::new());
                let result = thread::spawn(move || runner.run(tx, should_stop));

                while let Ok(event) = rx.recv() {
                    match event {
                        lib_io::runner::RunEvent::ProcessNewOutputLine(line) => print!("{}", line),
                        _ => info!("{:?}", event),
                    }
                }
                let result = result.join();
                info!("Result {:?}", result);
            }
        }
    }

    let mut join_handlers = Vec::new();
    for board in config.boards.iter() {
        let board = board.clone();
        join_handlers.push(thread::spawn(move || run_all_configs(board)));
    }

    for handler in join_handlers {
        let result = handler.join();
        info!("join result {:?}", result);
    }

    Ok(())
}

pub async fn handle_run(
    config_path: &PathBuf,
    server_url: &str,
    token: Option<String>,
) -> Result<()> {
    info!("Starting builder with config: {:?}", config_path);
    info!("Connecting to server: {}", server_url);

    let config = EjConfig::from_file(config_path)?;

    //config.validate()?;

    let auth_token = token.ok_or_else(|| {
        Error::Generic(String::from(
            "Builder token is required. Set EJB_TOKEN environment variable or use --token flag",
        ))
    })?;

    info!("Configuration loaded and validated");
    info!("Authentication token provided");
    todo!("TODO: Connect to server and start processing jobs");
}
