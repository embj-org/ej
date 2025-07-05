//! Unix socket communication module for the EJ Dispatcher Service.
//!
//! This module provides Unix socket-based communication for local administrative
//! operations that require elevated privileges or direct access to the dispatcher.
//! It handles:
//!
//! - Root user creation during initial setup
//! - Direct job dispatch for administrative tools
//! - Real-time job status updates
//! - Error handling and client connection management
//!
//! The socket interface is primarily used by the ejcli tool for setup and
//! testing operations that cannot be performed through the regular HTTP API.

use ej_dispatcher_sdk::ejsocket_message::{EjSocketClientMessage, EjSocketServerMessage};
use ej_models::auth::client_permission::{ClientPermission, NewClientPermission};
use ej_models::auth::permission::Permission;
use ej_models::client::ejclient::EjClient;
use ej_web::ejclient::create_client;
use ej_web::prelude::*;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::net::unix::OwnedWriteHalf;
use tokio::sync::mpsc::channel;
use tokio::task::JoinHandle;
use tracing::{error, info};

use crate::dispatcher::Dispatcher;

/// Sends a message to the Unix socket client.
///
/// This function serializes the response message to JSON and sends it
/// over the Unix socket connection, followed by a newline delimiter.
///
/// # Arguments
/// * `writer` - The write half of the Unix socket connection
/// * `response` - The server message to send to the client
///
/// # Returns
/// Result indicating success or failure of the send operation
///
/// # Errors
/// Returns an error if:
/// - JSON serialization fails
/// - Socket write operation fails
async fn send_message(writer: &mut OwnedWriteHalf, response: EjSocketServerMessage) -> Result<()> {
    info!("Socket Response {:?}", response);
    let serialized_response = serde_json::to_string(&response)?;
    writer.write_all(serialized_response.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    Ok(())
}

/// Handles incoming socket messages and dispatches them to appropriate handlers.
///
/// This function processes different types of client messages:
/// - `CreateRootUser`: Creates the initial administrative user with all permissions
/// - `Dispatch`: Submits a job for execution and streams status updates back
///
/// # Arguments
/// * `writer` - The write half of the socket for sending responses
/// * `message` - The parsed client message to handle
/// * `dispatcher` - Mutable reference to the dispatcher for job operations
///
/// # Returns
/// Result indicating success or failure of message handling
///
/// # Example Message Flow
/// ```text
/// Client -> CreateRootUser -> Server creates user -> CreateRootUserOk
/// Client -> Dispatch -> Server starts job -> DispatchOk -> JobUpdate...
/// ```
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
            let client = create_client(payload, &dispatcher.connection)?;

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
            send_message(writer, EjSocketServerMessage::CreateRootUserOk(client)).await?;
            Ok(())
        }
        EjSocketClientMessage::Dispatch { job, timeout } => {
            info!("Dispatching job {:?}", job);
            let (tx, mut rx) = channel(16);
            match dispatcher.dispatch_job(job, tx, timeout).await {
                Ok(job) => {
                    send_message(writer, EjSocketServerMessage::DispatchOk(job)).await?;
                    while let Some(msg) = rx.recv().await {
                        send_message(writer, EjSocketServerMessage::JobUpdate(msg)).await?;
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

/// Handles a single client connection to the Unix socket.
///
/// This function:
/// - Reads line-delimited JSON messages from the client
/// - Parses and handles each message through the message handler
/// - Sends error responses for parsing failures
/// - Manages the connection lifecycle until completion or error
///
/// # Arguments
/// * `dispatcher` - Dispatcher instance for handling job operations
/// * `stream` - The Unix socket stream for this client connection
///
/// # Returns
/// Result indicating success or failure of client handling
///
/// # Protocol
/// - Messages are JSON objects separated by newlines
/// - Each message receives a response before the next is processed
/// - Connection closes after message processing completes or on error
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

/// Sets up and starts the Unix socket server for administrative operations.
///
/// This function:
/// - Creates a Unix socket at `/tmp/ejd.sock`
/// - Starts a background task to accept connections
/// - Spawns individual handlers for each client connection
/// - Manages the socket lifecycle and error handling
///
/// # Arguments
/// * `dispatcher` - The dispatcher instance to clone for each client
///
/// # Returns
/// Result containing a JoinHandle for the socket server task
///
/// # Socket Usage
/// The socket is primarily used by:
/// - ejcli for initial setup and root user creation
/// - Administrative tools for direct job dispatch
/// - Testing utilities that need local access
///
/// # Example
/// ```rust
/// let socket_task = setup_socket(dispatcher).await?;
/// // Socket server runs in background
/// // Use ejcli or direct socket connection to communicate
/// ```
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
