#[derive(Debug, thiserror::Error, Clone)]
pub enum Error {
    #[error("Invalid Token")]
    InvalidToken,
    #[error("Token Missing")]
    TokenMissing,
    #[error("Token Expired")]
    TokenExpired,
    #[error(transparent)]
    TokenCreation(#[from] jsonwebtoken::errors::Error),

    #[error("Error hashing password {0}")]
    PasswordHash(argon2::password_hash::Error),
}
