#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Not enough arguments provided. Expected {0}. Got {1}")]
    MissingArgs(usize, usize),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
