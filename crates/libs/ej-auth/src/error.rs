//! Authentication error types.

/// Authentication errors.
#[derive(Debug, thiserror::Error, Clone)]
pub enum Error {
    /// JWT token is invalid or malformed.
    #[error("Invalid Token")]
    InvalidToken,

    /// Authentication token was not provided.
    #[error("Token Missing")]
    TokenMissing,

    /// JWT token has expired.
    #[error("Token Expired")]
    TokenExpired,

    /// JWT token creation or processing failed.
    #[error(transparent)]
    TokenCreation(#[from] jsonwebtoken::errors::Error),

    /// Password hashing operation failed.
    #[error("Error hashing password {0}")]
    PasswordHash(argon2::password_hash::Error),
}
