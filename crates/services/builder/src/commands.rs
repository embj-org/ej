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
    id: Option<String>,
    token: Option<String>,
) -> Result<()> {
    info!("Starting builder with config: {:?}", config_path);
    info!("Connecting to server: {}", server_url);

    let config = EjConfig::from_file(config_path)?;

    let id =
        uuid::Uuid::from_str(&id.or_else(|| std::env::var("EJB_ID").ok()).ok_or_else(|| {
            Error::Generic(String::from(
                "Builder token is required. Set EJB_ID environment variable or use --id flag",
            ))
        })?)
        .map_err(|err| Error::Generic(format!("Failed to parse id to uuid ({})", err)))?;

    let auth_token = token
        .or_else(|| std::env::var("EJB_TOKEN").ok())
        .ok_or_else(|| {
            Error::Generic(String::from(
                "Builder token is required. Set EJB_TOKEN environment variable or use --token flag",
            ))
        })?;

    let client = ApiClient::new(server_url);
    let builder_api = EjBuilderApi {
        id,
        token: auth_token.clone(),
    };

    let body = serde_json::to_string(&builder_api)?;
    let response: EjBuilderApi = client
        .post("v1/builder/login", body)
        .await
        .expect("Failed to login");

    info!("Successfully logged in as builder {}", response.id);

    let ws_url = if server_url.starts_with("https") {
        server_url.replace("https", "ws")
    } else {
        assert!(server_url.starts_with("http"));
        server_url.replace("http", "ws")
    };

    let ws_url = format!("{}/v1/builder/ws", ws_url);
    debug!("Connecting to WebSocket: {}", ws_url);

    let mut request = ws_url
        .into_client_request()
        .expect("Failed to create client websocket request");

    request.headers_mut().insert(
        AUTH_HEADER,
        format!("{}{}", AUTH_HEADER_PREFIX, response.token)
            .parse()
            .unwrap(),
    );

    let (ws_stream, _) = connect_async(request)
        .await
        .map_err(|e| Error::Generic(format!("Failed to connect to WebSocket: {}", e)))?;

    info!("WebSocket connection established");

    let (mut write, mut read) = ws_stream.split();

    let config_message = serde_json::to_string(&config)
        .map_err(|e| Error::Generic(format!("Failed to serialize config: {}", e)))?;

    write
        .send(Message::Text(config_message.into()))
        .await
        .map_err(|e| Error::Generic(format!("Failed to send config: {}", e)))?;

    info!("Configuration sent to server");

    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                debug!("Received message: {}", text);

                let server_message: EjServerMessage = match serde_json::from_str(&text) {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("Failed to parse server message: {}", e);
                        continue;
                    }
                };

                match server_message {
                    EjServerMessage::Run(job) => {
                        let validation_result = handle_validate(config_path);
                        match validation_result {
                            Ok(_) => {
                                info!("Job validation successful");
                                let success_message = EjClientMessage::JobSucess;
                                let response =
                                    serde_json::to_string(&success_message).map_err(|e| {
                                        Error::Generic(format!(
                                            "Failed to serialize response: {}",
                                            e
                                        ))
                                    })?;

                                if let Err(e) = write.send(Message::Text(response.into())).await {
                                    error!("Failed to send success response: {}", e);
                                }
                            }
                            Err(e) => {
                                error!("Job validation failed: {}", e);
                                let failure_message = EjClientMessage::JobFailure;
                                let response =
                                    serde_json::to_string(&failure_message).map_err(|e| {
                                        Error::Generic(format!(
                                            "Failed to serialize response: {}",
                                            e
                                        ))
                                    })?;

                                if let Err(e) = write.send(Message::Text(response.into())).await {
                                    error!("Failed to send failure response: {}", e);
                                }
                            }
                        }
                    }
                    EjServerMessage::Close => {
                        info!("Received close command from server");
                        break;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket connection closed by server");
                break;
            }
            Ok(Message::Ping(data)) => {
                debug!("Received ping, sending pong");
                if let Err(e) = write.send(Message::Pong(data)).await {
                    error!("Failed to send pong: {}", e);
                }
            }
            Ok(Message::Pong(_)) => {
                debug!("Received pong");
            }
            Ok(Message::Binary(_)) => {
                warn!("Received unexpected binary message");
            }
            Ok(Message::Frame(_)) => {
                debug!("Received raw frame message");
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    info!("Builder shutting down");
    Ok(())
}
