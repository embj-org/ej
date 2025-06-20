use ej::ej_builder::api::EjBuilderApi;
use ej::ej_config::ej_board::EjBoard;
use ej::ej_config::ej_config::EjConfig;
use ej::ej_message::{EjClientMessage, EjServerMessage};
use ej::prelude::*;
use ej::web::ctx::{AUTH_HEADER, AUTH_HEADER_PREFIX};
use futures_util::{SinkExt, StreamExt};
use lib_io::runner::Runner;
use lib_requests::ApiClient;
use serde_json;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{debug, error, info, warn};
use uuid;

pub fn handle_parse(config_path: &PathBuf) -> Result<()> {
    println!("Parsing configuration file: {:?}", config_path);

    let config = EjConfig::from_file(config_path)?;

    println!("Configuration parsed successfully");
    println!("Global version: {}", config.global.version);
    println!("Number of boards: {}", config.boards.len());

    for (board_idx, board) in config.boards.iter().enumerate() {
        println!("\nBoard {}: {}", board_idx + 1, board.name);
        println!("  Description: {}", board.description);
        println!("  Configurations: {}", board.configs.len());

        for (config_idx, board_config) in board.configs.iter().enumerate() {
            println!("    Config {}: {}", config_idx + 1, board_config.name);
            println!("      Tags: {:?}", board_config.tags);
            println!("      Build script: {:?}", board_config.build_script);
            println!("      Run script: {:?}", board_config.run_script);
            println!("      Results path: {:?}", board_config.results_path);
            println!("      Library path: {:?}", board_config.library_path);
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

    let config = EjConfig::from_file(config_path)?;
}
