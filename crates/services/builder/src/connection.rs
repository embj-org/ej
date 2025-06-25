use std::path::PathBuf;
use std::str::FromStr;

use ej::ej_config::ej_config::{EjConfig, EjDispatcherConfig};
use ej::ej_job::api::{EjBuildResult, EjRunOutput, EjRunResult};
use ej::ej_message::EjServerMessage;
use ej::prelude::*;
use ej::web::ctx::AUTH_HEADER_PREFIX;
use ej::{ej_builder::api::EjBuilderApi, web::ctx::AUTH_HEADER};
use futures_util::{SinkExt, StreamExt};
use lib_requests::ApiClient;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use crate::build::build;
use crate::logs::dump_logs_to_temporary_file;
use crate::run::run;

pub async fn handle_connect(
    config_path: &PathBuf,
    server_url: &str,
    id: Option<String>,
    token: Option<String>,
) -> Result<()> {
    info!("Starting builder with config: {:?}", config_path);

    info!("Connecting to server: {}", server_url);
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
        .post_and_deserialize("v1/builder/login", body)
        .await
        .expect("Failed to login");

    info!("Successfully logged in as builder {}", builder.id);
    let body = serde_json::to_string(&config)?;
    let dispatcher_config: EjDispatcherConfig = client
        .post_and_deserialize("v1/builder/config", body)
        .await
        .expect("Failed to push config");
    info!("Successfully pushed config");

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

    info!("WebSocket connection established");

    let (mut write, mut read) = ws_stream.split();

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
                        let mut output = EjRunOutput::new(&config);
                        let result = build(&config, &mut output);
                        if let Err(err) = dump_logs_to_temporary_file(&output) {
                            error!("Failed to dump logs to file - {err}");
                        }
                        let response = EjBuildResult {
                            job_id: job.id,
                            builder_id: builder.id,
                            config: dispatcher_config.clone(),
                            logs: output.logs,
                            successful: result.is_ok(),
                        };

                        let body = serde_json::to_string(&response);
                        match body {
                            Ok(body) => match client.post("v1/builder/build_result", body).await {
                                Ok(_) => trace!("Run results sent"),
                                Err(err) => {
                                    /* TODO: Store the results locally to send them later */
                                    error!("Failed to send build results {err}");
                                }
                            },
                            Err(err) => {
                                error!("Failed to serialize run results {}", err);
                            }
                        };
                    }
                    EjServerMessage::Run(job) => {
                        let mut output = EjRunOutput::new(&config);
                        let result = run(&config, &mut output);
                        if let Err(err) = dump_logs_to_temporary_file(&output) {
                            error!("Failed to dump logs to file - {err}");
                        }
                        let response = EjRunResult {
                            job_id: job.id,
                            builder_id: builder.id,
                            config: dispatcher_config.clone(),
                            logs: output.logs,
                            results: output.results,
                            successful: result.is_ok(),
                        };
                        let body = serde_json::to_string(&response);
                        match body {
                            Ok(body) => match client.post("v1/builder/run_result", body).await {
                                Ok(_) => trace!("Run results sent"),
                                Err(err) => {
                                    /* TODO: Store the results locally to send them later */
                                    error!("Failed to send run results {err}");
                                }
                            },
                            Err(err) => {
                                error!("Failed to serialize run results {}", err);
                            }
                        }
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
