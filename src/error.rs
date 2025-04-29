//! Main Crate Error

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Generic {0}")]
    Generic(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    JWT(#[from] jsonwebtoken::errors::Error),

    #[error("PasswordHash {0}")]
    PasswordHash(argon2::password_hash::Error),

    #[error(transparent)]
    R2D2(#[from] diesel::r2d2::PoolError),

    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),

    /* Api Errors */
    #[error("API Forbidden")]
    ApiForbidden,

    /* Auth Errors */
    #[error("Auth Token Missing")]
    AuthTokenMissing,
    #[error("Auth Token Expired")]
    AuthTokenExpired,
    #[error("Invalid Token")]
    AuthInvalidToken,
    #[error("Auth Token Creation")]
    AuthTokenCreation,
    #[error("Wrong Credentials")]
    WrongCredentials,
    #[error("Missing Credentials")]
    MissingCredentials,

    #[error("Context Missing")]
    CtxMissing,
}
