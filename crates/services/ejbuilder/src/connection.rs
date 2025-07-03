use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use ej::ej_config::ej_config::EjConfig;
use ej::ej_job::api::EjJobCancelReason;
use ej::ej_job::results::api::{EjBuilderBuildResult, EjBuilderRunResult, EjRunOutput};
use ej::ej_message::EjServerMessage;
use ej::prelude::*;
use ej::web::ctx::AUTH_HEADER_PREFIX;
use ej::{ej_builder::api::EjBuilderApi, web::ctx::AUTH_HEADER};
use futures_util::{SinkExt, StreamExt};
use lib_builder_sdk::BuilderEvent;
use lib_requests::ApiClient;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use crate::build::build;
use crate::builder::Builder;
use crate::checkout::checkout_all;
use crate::logs::dump_logs_to_temporary_file;
use crate::run::run;

pub async fn handle_connect(
    builder: Builder,
    server_url: &str,
    id: Option<String>,
    token: Option<String>,
) -> Result<()> {
    info!("Starting builder with config: {:?}", builder.config_path);

    info!("Connecting to server: {}", server_url);
    let config = &builder.config;

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
    let builder_api = EjBuilderApi {
        id,
        token: auth_token.clone(),
    };

    let body = serde_json::to_string(&builder_api)?;
    let builder_api: EjBuilderApi = client
        .post_and_deserialize("v1/builder/login", body)
        .await
        .expect("Failed to login");

    info!("Successfully logged in as builder {}", builder_api.id);
    let body = serde_json::to_string(&config)?;
    let config: EjConfig = client
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
        format!("{}{}", AUTH_HEADER_PREFIX, builder_api.token)
            .parse()
            .unwrap(),
    );

    let (ws_stream, _) = connect_async(request)
        .await
        .map_err(|e| Error::Generic(format!("Failed to connect to WebSocket: {}", e)))?;

    info!("WebSocket connection established");

    let (mut write, mut read) = ws_stream.split();

    let mut current_job: Option<(Uuid, JoinHandle<()>, Arc<AtomicBool>)> = None;
    let config = Arc::new(config);
    let builder = Arc::new(builder);
    let client = Arc::new(client);

    while let Some(message) = read.next().await {
        if let Some(ref job) = current_job {
            if job.1.is_finished() {
                current_job = None;
            }
        }

        match message {
            Ok(Message::Text(text)) => {
                info!("Received message: {}", text);

                let server_message: EjServerMessage = match serde_json::from_str(&text) {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("Failed to parse server message: {}", e);
                        continue;
                    }
                };

                match server_message {
                    EjServerMessage::Build(job) => {
                        if let Some(job) = current_job.take() {
                            warn!(
                                "Received a new build request while a job is happening. Cancelling it"
                            );
                            cancel_job(&builder, &job.0, job.1, job.2, EjJobCancelReason::Timeout)
                                .await;
                        }

                        let config = Arc::clone(&config);
                        let builder = Arc::clone(&builder);
                        let client = Arc::clone(&client);
                        let stop = Arc::new(AtomicBool::new(false));
                        let t_stop = Arc::clone(&stop);

                        let handle = tokio::spawn(async move {
                            let mut output = EjRunOutput::new(&config);
                            let mut result = checkout_all(
                                &config,
                                &job.commit_hash,
                                &job.remote_url,
                                job.remote_token,
                                &mut output,
                            );
                            if result.is_ok() {
                                result = build(&builder, &config, &mut output, t_stop);
                            }
                            if let Err(err) = dump_logs_to_temporary_file(&output) {
                                error!("Failed to dump logs to file - {err}");
                            }
                            let response = EjBuilderBuildResult {
                                job_id: job.id,
                                builder_id: builder_api.id,
                                logs: output.logs,
                                successful: result.is_ok(),
                            };

                            let body = serde_json::to_string(&response);
                            match body {
                                Ok(body) => {
                                    match client.post("v1/builder/build_result", body).await {
                                        Ok(response) => info!("Build results sent {:?}", response),
                                        Err(err) => {
                                            /* TODO: Store the results locally to send them later */
                                            error!("Failed to send build results {err}");
                                        }
                                    }
                                }
                                Err(err) => {
                                    error!(
                                        "Failed to serialize {:?} run results {}",
                                        response, err
                                    );
                                }
                            };
                        });
                        current_job = Some((job.id.clone(), handle, stop));
                    }
                    EjServerMessage::BuildAndRun(job) => {
                        if let Some(job) = current_job.take() {
                            warn!(
                                "Received a new build request while a job is happening. Cancelling it"
                            );
                            cancel_job(&builder, &job.0, job.1, job.2, EjJobCancelReason::Timeout)
                                .await;
                        }
                        let config = Arc::clone(&config);
                        let builder = Arc::clone(&builder);
                        let client = Arc::clone(&client);
                        let stop = Arc::new(AtomicBool::new(false));
                        let t_stop = Arc::clone(&stop);
                        let handle = tokio::spawn(async move {
                            let mut output = EjRunOutput::new(&config);
                            let mut result = checkout_all(
                                &config,
                                &job.commit_hash,
                                &job.remote_url,
                                job.remote_token,
                                &mut output,
                            );
                            if result.is_ok() {
                                result = build(&builder, &config, &mut output, Arc::clone(&t_stop));
                            }
                            if result.is_ok() {
                                result = run(&builder, &config, &mut output, t_stop);
                            }
                            if let Err(err) = dump_logs_to_temporary_file(&output) {
                                error!("Failed to dump logs to file - {err}");
                            }
                            let response = EjBuilderRunResult {
                                job_id: job.id,
                                builder_id: builder_api.id,
                                logs: output.logs,
                                results: output.results,
                                successful: result.is_ok(),
                            };
                            let body = serde_json::to_string(&response);
                            match body {
                                Ok(body) => {
                                    match client.post("v1/builder/run_result", body).await {
                                        Ok(_) => trace!("Run results sent"),
                                        Err(err) => {
                                            /* TODO: Store the results locally to send them later */
                                            error!("Failed to send run results {err}");
                                        }
                                    }
                                }
                                Err(err) => {
                                    error!("Failed to serialize run results {}", err);
                                }
                            }
                        });
                        current_job = Some((job.id.clone(), handle, stop));
                    }
                    EjServerMessage::Cancel(reason, job_id) => {
                        if let Some(curr_job) = current_job.take() {
                            if curr_job.0 == job_id {
                                cancel_job(&builder, &curr_job.0, curr_job.1, curr_job.2, reason)
                                    .await;
                            } else {
                                warn!(
                                    "Received cancel request for a job different than the one in progress. "
                                )
                            }
                        } else {
                            info!("Received cancel request but no job is currently in progress. ")
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
async fn cancel_job(
    builder: &Builder,
    job_id: &Uuid,
    mut handle: JoinHandle<()>,
    stop: Arc<AtomicBool>,
    reason: EjJobCancelReason,
) {
    info!("Cancelling {job_id} - Reason: {reason}");

    // This sends a message to the child process to exit
    if let Err(err) = builder.tx.send(BuilderEvent::Exit).await {
        error!("Failed to send exit request to builder task - {err}");
    }

    // Ideally, the child process finishes its execution by itself and its task handler will finish
    let timeout_result = timeout(Duration::from_secs(60), &mut handle).await;

    match timeout_result {
        Ok(Ok(())) => {
            info!("Job {job_id} completed gracefully");
        }
        Ok(Err(join_err)) => {
            warn!("Task handling {job_id} finished with error: {join_err}");
        }
        Err(_timeout) => {
            error!(
                "Process taking care of {job_id} did not complete within timeout, forcing it to exit. \
                This can cause problems in future runs. \
                EJ recommends using its builder sdk to handle these cases for you. \
                If you're already using it, make sure you handle the exit message correctly"
            );
            stop.store(true, Ordering::Relaxed);
            let timeout_result = timeout(Duration::from_secs(30), &mut handle).await;

            match timeout_result {
                Ok(result) => {
                    info!("Task handling process finished {:?}", result);
                }
                Err(_timeout) => {
                    warn!(
                        "Even after force stopping the process, the task handling it didn't complete in time. Aborting. \
                        This may mean a task will be left in zombie state"
                    );
                    handle.abort();
                    let result = handle.await;
                    info!("Task result after aborting {:?}", result);
                }
            }
        }
    }
}
