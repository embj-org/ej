use ej::ej_client::db::EjClient;
use ej::ej_client_permission::{ClientPermission, NewClientPermission};
use ej::ej_message::{EjSocketClientMessage, EjSocketServerMessage};
use ej::permission::Permission;
use ej::prelude::*;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::task::JoinHandle;
use tracing::{error, info};

use crate::dispatcher::Dispatcher;

async fn handle_message(
    message: EjSocketClientMessage,
    dispatcher: &mut Dispatcher,
) -> Result<(EjSocketServerMessage, bool)> {
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
            Ok((EjSocketServerMessage::CreateRootUserOk(client), true))
        }
        EjSocketClientMessage::Dispatch(job) => {
            info!("Dispatching job {:?}", job);
            match dispatcher.dispatch_job(job).await {
                Ok(job) => Ok((EjSocketServerMessage::DispatchOk(job), false)),
                Err(err) => {
                    error!("Failed to dispatch job - {}", err);
                    Ok((EjSocketServerMessage::Error(err.to_string()), true))
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
                    let (response, close, err) =
                        match handle_message(message, &mut dispatcher).await {
                            Ok((response, close)) => (response, close, None),
                            Err(err) => (
                                EjSocketServerMessage::Error(err.to_string()),
                                true,
                                Some(err),
                            ),
                        };
                    info!("Socket Response {:?} Error: {:?}", response, err);
                    let serialized_response = serde_json::to_string(&response)?;
                    writer.write_all(serialized_response.as_bytes()).await?;
                    writer.write_all(b"\n").await?;
                    if close {
                        if let Some(err) = err {
                            return Err(err);
                        } else {
                            return Ok(());
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
