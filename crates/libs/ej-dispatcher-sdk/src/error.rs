//! Dispatcher SDK error types.

use crate::ejsocket_message::EjSocketServerMessage;

/// Dispatcher SDK errors.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Build operation failed.
    #[error("Build Error")]
    BuildError,
    /// Run operation failed.
    #[error("Run Error")]
    RunError,

    /// Unexpected Socket Message
    #[error("Unexpected message from socket")]
    UnexpectedSocketMessage(EjSocketServerMessage),

    /// I/O operation failed.
    #[error(transparent)]
    IO(#[from] std::io::Error),

    /// JSON serialization/deserialization failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
