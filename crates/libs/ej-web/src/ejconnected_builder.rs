//! Connected builder management for WebSocket communication.

use std::net::SocketAddr;

use ej_dispatcher_sdk::ejws_message::EjWsServerMessage;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::ctx::ctx_client::CtxClient;

/// Represents a builder that is currently connected via WebSocket.
#[derive(Debug, Clone)]
pub struct EjConnectedBuilder {
    /// The builder's client context.
    pub builder: CtxClient,
    /// Message sender for WebSocket communication.
    pub tx: Sender<EjWsServerMessage>,
    /// The builder's network address.
    pub addr: SocketAddr,
    /// Connection ID
    pub connection_id: Uuid,
}
