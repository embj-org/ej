use ej::ej_message::{EjServerMessage, EjSocketMessage};
use ej::prelude::*;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::task::JoinHandle;
use tracing::info;

use crate::dispatcher::Dispatcher;

async fn handle_client(dispatcher: Dispatcher, stream: UnixStream) -> Result<()> {
    info!("Connected to socket client");
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await? {
            0 => break,
            _ => {
                line.pop();

                if let Ok(msg) = serde_json::from_str::<EjSocketMessage>(&line) {
                    match msg {
                        EjSocketMessage::CreateUser(payload) => {
                            info!("Creating user {}", payload.name);
                            let response = payload.persist(&dispatcher.connection)?;
                            let serialized_response = serde_json::to_string(&response)?;
                            writer.write_all(serialized_response.as_bytes()).await?;
                            writer.write_all(b"\n").await?;
                            break;
                        }
                        EjSocketMessage::Dispatch(job) => {
                            info!("Dispatching job {:?}", job);
                            let builders = dispatcher.builders.lock().await;
                            for builder in builders.iter() {
                                let message = EjServerMessage::Run(job.clone());
                                if let Err(err) = builder.tx.send(message).await {
                                    tracing::error!(
                                        "Failed to dispatch builder {:?} - {err}",
                                        builder
                                    );
                                }
                            }
                        }
                    }
                } else {
                    tracing::warn!("Failed to parse message: {}", line);
                    break;
                }
            }
        }
    }
    Ok(())
}

pub async fn setup_socket(dispatcher: Dispatcher) -> Result<JoinHandle<Result<()>>> {
    let socket_path = "/tmp/ejd.sock";
    let _ = std::fs::remove_file(socket_path);

    let listener = tokio::net::UnixListener::bind(socket_path)?;
    tracing::debug!("Socket listening on {}", socket_path);

    let handler = tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let dispatcher = dispatcher.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_client(dispatcher, stream).await {
                            tracing::error!("Error handling client: {}", e);
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to accept connection: {}", e);
                }
            }
        }
    });
    Ok(handler)
}
