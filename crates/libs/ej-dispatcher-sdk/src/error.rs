//! Main Crate Error

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Build Error")]
    BuildError,
    #[error("Run Error")]
    RunError,

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
