//! Error types for the builder SDK.

/// Builder SDK errors.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Not enough command line arguments provided.
    #[error("Not enough arguments provided. Expected {0}. Got {1}")]
    MissingArgs(usize, usize),

    /// Invalid argument
    #[error("Invalid action {0}")]
    InvalidAction(String),

    /// I/O operation failed.
    #[error(transparent)]
    IO(#[from] std::io::Error),

    /// JSON serialization/deserialization failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
