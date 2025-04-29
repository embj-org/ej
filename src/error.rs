//! Main Crate Error

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Generic {0}")]
    Generic(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    JWT(#[from] jsonwebtoken::errors::Error),

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

    #[error("Context Missing")]
    CtxMissing,
}
