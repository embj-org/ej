//! Main Crate Error

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Generic {0}")]
    Generic(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Models(#[from] lib_models::error::Error),

    #[error(transparent)]
    JWT(#[from] jsonwebtoken::errors::Error),

    #[error("PasswordHash {0}")]
    PasswordHash(argon2::password_hash::Error),

    #[error(transparent)]
    Toml(#[from] toml::de::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("Invalid Job Type")]
    InvalidJobType,

    #[error("Internal task communication channel error")]
    ChannelSendError,

    /* Builder Errors */
    #[error("Build error")]
    BuildError,

    #[error("No builders available")]
    NoBuildersAvailable,

    /* Run Errors */
    #[error("Run error")]
    RunError,

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
