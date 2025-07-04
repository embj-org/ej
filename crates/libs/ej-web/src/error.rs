//! Main Crate Error

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;
use tracing::error;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Generic {0}")]
    Generic(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Models(#[from] ej_models::error::Error),

    #[error(transparent)]
    Auth(#[from] ej_auth::error::Error),

    #[error(transparent)]
    Toml(#[from] toml::de::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("Internal error dispatching job")]
    InternalErrorDispatchingJob,

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

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        error!("Creating API error response for error: {:?}", self);
        let (status, message) = match self {
            Error::AuthTokenMissing => (StatusCode::UNAUTHORIZED, "Authentication required"),
            Error::AuthTokenExpired => (StatusCode::UNAUTHORIZED, "Authentication token expired"),
            Error::AuthInvalidToken => (StatusCode::UNAUTHORIZED, "Invalid authentication token"),
            Error::WrongCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            Error::MissingCredentials => (StatusCode::UNAUTHORIZED, "Missing credentials"),
            Error::ApiForbidden => (StatusCode::FORBIDDEN, "Access forbidden"),
            Error::InvalidJobType => (StatusCode::BAD_REQUEST, "Invalid job type"),
            Error::NoBuildersAvailable => (StatusCode::NOT_FOUND, "No builders available"),
            Error::AuthTokenCreation
            | Error::Generic(_)
            | Error::IO(_)
            | Error::InternalErrorDispatchingJob
            | Error::CtxMissing
            | Error::Json(_)
            | Error::Toml(_)
            | Error::BuildError
            | Error::RunError
            | Error::ChannelSendError
            | Error::Auth(_)
            | Error::Models(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(json!({
            "error": {
                "message": message,
                "status": status.as_u16()
            }
        }));
        (status, body).into_response()
    }
}
