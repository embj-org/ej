use crate::dispatcher::DispatcherEvent;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::error::Error),

    #[error(transparent)]
    DispatcherEventSendError(#[from] tokio::sync::mpsc::error::SendError<DispatcherEvent>),

    #[error(transparent)]
    Config(#[from] ej_config::error::Error),

    #[error(transparent)]
    Model(#[from] ej_models::error::Error),

    #[error(transparent)]
    Web(#[from] ej_web::error::Error),

    #[error(transparent)]
    Uuid(#[from] uuid::Error),

    #[error(transparent)]
    TokioTungstenite(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("No builders available")]
    NoBuildersAvailable,

    #[error("Failed to receive WebSocket Message")]
    WsSocketReceiveFail,

    #[error("WebSocket Receive Error {0}")]
    WsSocketReceiveError(String),

    #[error("Invalide WebSocket Message")]
    InvalidWsMessage,

    #[error("WebSocket Receive Error {0}")]
    Axum(#[from] axum::Error),
}
