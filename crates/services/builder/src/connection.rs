use std::path::PathBuf;
use std::str::FromStr;

use ej::ej_config::ej_config::EjConfig;
use ej::ej_job::api::EjDeployableJob;
use ej::ej_message::{EjClientMessage, EjServerMessage};
use ej::prelude::*;
use ej::web::ctx::AUTH_HEADER_PREFIX;
use ej::{ej_builder::api::EjBuilderApi, web::ctx::AUTH_HEADER};
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use lib_requests::ApiClient;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, warn};
use uuid::Uuid;

use crate::build::build;
use crate::commands::handle_run_and_build;
use crate::run::run;

pub async fn handle_connect(
    config_path: &PathBuf,
    server_url: &str,
    id: Option<String>,
    token: Option<String>,
) -> Result<()> {
    println!("Starting builder with config: {:?}", config_path);

    println!("Connecting to server: {}", server_url);
    let config = EjConfig::from_file(config_path)?;

    let id = Uuid::from_str(&id.or_else(|| std::env::var("EJB_ID").ok()).ok_or_else(|| {
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
    let builder = EjBuilderApi {
        id,
        token: auth_token.clone(),
    };

    let body = serde_json::to_string(&builder)?;
    let builder: EjBuilderApi = client
        .post("v1/builder/login", body)
        .await
        .expect("Failed to login");

    println!("Successfully logged in as builder {}", builder.id);

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
        format!("{}{}", AUTH_HEADER_PREFIX, builder.token)
            .parse()
            .unwrap(),
    );

    let (ws_stream, _) = connect_async(request)
        .await
        .map_err(|e| Error::Generic(format!("Failed to connect to WebSocket: {}", e)))?;

    println!("WebSocket connection established");

    let (mut write, mut read) = ws_stream.split();

    let config_message = serde_json::to_string(&config)
        .map_err(|e| Error::Generic(format!("Failed to serialize config: {}", e)))?;

    write
        .send(Message::Text(config_message.into()))
        .await
        .map_err(|e| Error::Generic(format!("Failed to send config: {}", e)))?;

    println!("Configuration sent to server");

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
                    EjServerMessage::Build(job) => {
                        let response = match build(&config) {
                            Ok(()) => EjClientMessage::BuildSuccess {
                                job_id: job.id,
                                builder_id: builder.id,
                            },
                            Err(err) => EjClientMessage::BuildFailure {
                                job_id: job.id,
                                builder_id: builder.id,
                                error: err.to_string(),
                            },
                        };

                        let payload = serde_json::to_string(&response);

                        match payload {
                            Ok(payload) => match write.send(payload.into()).await {
                                Ok(()) => debug!("Message sent to server {:?}", response),
                                Err(err) => error!("Failed to send message to server {}", err),
                            },
                            Err(err) => error!("Failed to serialize payload {}", err),
                        }
                    }
                    EjServerMessage::Run(job) => {
                        let response = match run(&config) {
                            Ok(()) => EjClientMessage::RunSuccess {
                                job_id: job.id,
                                builder_id: builder.id,
                            },
                            Err(err) => EjClientMessage::RunFailure {
                                job_id: job.id,
                                builder_id: builder.id,
                                error: err.to_string(),
                            },
                        };

                        let payload = serde_json::to_string(&response);

                        match payload {
                            Ok(payload) => match write.send(payload.into()).await {
                                Ok(()) => debug!("Message sent to server {:?}", response),
                                Err(err) => error!("Failed to send message to server {}", err),
                            },
                            Err(err) => error!("Failed to serialize payload {}", err),
                        }
                    }
                    EjServerMessage::Error(err) => {
                        println!("Server error {err}");
                        break;
                    }
                    EjServerMessage::Close => {
                        println!("Received close command from server");
                        break;
                    }
                };
            }
            Ok(Message::Close(_)) => {
                println!("WebSocket connection closed by server");
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

    println!("Builder shutting down");
    Ok(())
}

async fn serialize_and_send_to_server(writer: SplitSink, message: EjClientMessage) {}
