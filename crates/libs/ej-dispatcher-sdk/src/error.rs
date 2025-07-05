//! Dispatcher SDK error types.

/// Dispatcher SDK errors.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Build operation failed.
    #[error("Build Error")]
    BuildError,
    /// Run operation failed.
    #[error("Run Error")]
    RunError,

    /// I/O operation failed.
    #[error(transparent)]
    IO(#[from] std::io::Error),

    /// JSON serialization/deserialization failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
