use ej::ej_client::db::EjClient;
use ej::ej_client_permission::{ClientPermission, NewClientPermission};

use ej::ej_job::api::EjJobUpdate;
use ej::ej_message::{EjSocketClientMessage, EjSocketServerMessage};
use ej::permission::Permission;
use ej::prelude::*;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::net::unix::OwnedWriteHalf;
use tokio::sync::mpsc::channel;
use tokio::task::JoinHandle;
use tracing::{error, info};

use crate::dispatcher::Dispatcher;

async fn send_message(writer: &mut OwnedWriteHalf, response: EjSocketServerMessage) -> Result<()> {
    info!("Socket Response {:?}", response);
    let serialized_response = serde_json::to_string(&response)?;
    writer.write_all(serialized_response.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    Ok(())
}

async fn handle_message(
    writer: &mut OwnedWriteHalf,
    message: EjSocketClientMessage,
    dispatcher: &mut Dispatcher,
) -> Result<()> {
    match message {
        EjSocketClientMessage::CreateRootUser(payload) => {
            let clients = EjClient::fetch_all(&dispatcher.connection)?;
            if clients.len() > 0 {
                error!("Tried to create root user but it already exists");
                return Err(Error::ApiForbidden);
            }
            info!("Creating root user {}", payload.name);
            let client = payload.persist(&dispatcher.connection)?;

            let permissions = Permission::fetch_all(&dispatcher.connection)?;
            for permission in permissions.iter() {
                let client_permission = NewClientPermission {
                    ejclient_id: client.id,
                    permission_id: permission.id.clone(),
                };
                let client_permission =
                    ClientPermission::new(&dispatcher.connection, client_permission);
                if let Err(err) = client_permission {
                    error!("Failed to add permission {} to user {}", permission.id, err);
                }
            }
            send_message(writer, EjSocketServerMessage::CreateRootUserOk(client));
            Ok(())
        }
        EjSocketClientMessage::Dispatch(job) => {
            info!("Dispatching job {:?}", job);
            let (tx, mut rx) = channel(16);
            match dispatcher.dispatch_job(job, tx).await {
                Ok(job) => {
                    send_message(writer, EjSocketServerMessage::DispatchOk(job)).await?;
                    while let Some(msg) = rx.recv().await {
                        let end = matches!(msg, EjJobUpdate::JobFinished);
                        send_message(writer, EjSocketServerMessage::JobUpdate(msg)).await?;
                        if end {
                            return Ok(());
                        }
                    }
                    Ok(())
                }
                Err(err) => {
                    error!("Failed to dispatch job - {}", err);
                    send_message(writer, EjSocketServerMessage::Error(err.to_string())).await?;
                    Ok(())
                }
            }
        }
    }
}

async fn handle_client(mut dispatcher: Dispatcher, stream: UnixStream) -> Result<()> {
    info!("Connected to socket client");
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await? {
            0 => break,
            _ => {
                line.pop();
                if let Ok(message) = serde_json::from_str::<EjSocketClientMessage>(&line) {
                    info!("Socket Message {:?}", message);
                    match handle_message(&mut writer, message, &mut dispatcher).await {
                        Ok(_) => {
                            return Ok(());
                        }
                        Err(err) => {
                            error!("Error during socket message handling  - {err}");
                            send_message(
                                &mut writer,
                                EjSocketServerMessage::Error(err.to_string()),
                            )
                            .await?;
                            return Err(err);
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
