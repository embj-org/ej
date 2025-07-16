//! Error types for the EJ Builder Service.
//!
//! Defines error variants that can occur during builder operations,
//! including configuration errors, build failures, and connection issues.

/// Errors that can occur in the EJ Builder Service.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::error::Error),

    #[error("Checkout Error")]
    CheckoutError,

    #[error("Build Error")]
    BuildError,

    #[error("Builder ID is missing. Set EJB_ID environment variable or use --id cli argument")]
    BuilderIDMissing,

    #[error(
        "Builder Token is missing. Set EJB_TOKEN environment variable or use --token cli argument"
    )]
    BuilderTokenMissing,

    #[error(transparent)]
    ThreadJoin(#[from] tokio::task::JoinError),

    #[error("Failed to get exit status from process")]
    ProcessExitStatusUnavailable,

    #[error(transparent)]
    Config(#[from] ej_config::error::Error),

    #[error(transparent)]
    Uuid(#[from] uuid::Error),

    #[error(transparent)]
    TokioTungstenite(#[from] tokio_tungstenite::tungstenite::Error),
}
