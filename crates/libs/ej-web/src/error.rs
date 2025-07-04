//! Main Crate Error

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;
use tracing::error;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Models(#[from] ej_models::error::Error),

    #[error(transparent)]
    Auth(#[from] ej_auth::error::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("Internal error dispatching job")]
    InternalErrorDispatchingJob,

    #[error("Invalid Job Type")]
    InvalidJobType,

    #[error("No builders available")]
    NoBuildersAvailable,

    /* Api Errors */
    #[error("API Forbidden")]
    ApiForbidden,

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
            Error::WrongCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            Error::MissingCredentials | Error::CtxMissing => {
                (StatusCode::UNAUTHORIZED, "Missing credentials")
            }
            Error::ApiForbidden => (StatusCode::FORBIDDEN, "Access forbidden"),
            Error::InvalidJobType => (StatusCode::BAD_REQUEST, "Invalid job type"),
            Error::NoBuildersAvailable => (StatusCode::NOT_FOUND, "No builders available"),
            Error::Auth(err) => match err {
                ej_auth::error::Error::InvalidToken => {
                    (StatusCode::UNAUTHORIZED, "Invalid authentication token")
                }
                ej_auth::error::Error::TokenMissing => {
                    (StatusCode::UNAUTHORIZED, "Authentication required")
                }
                ej_auth::error::Error::TokenExpired => {
                    (StatusCode::UNAUTHORIZED, "Authentication token expired")
                }
                ej_auth::error::Error::TokenCreation(_)
                | ej_auth::error::Error::PasswordHash(_) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
                }
            },
            Error::AuthTokenCreation
            | Error::IO(_)
            | Error::InternalErrorDispatchingJob
            | Error::Json(_)
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
