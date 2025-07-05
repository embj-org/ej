//! Main crate error types for ej-web.
//!
//! This module defines the error types used throughout the ej-web library,
//! including HTTP response mapping for API endpoints.

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;
use tracing::error;

/// Main error type for the ej-web library.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// I/O operation failed.
    #[error(transparent)]
    IO(#[from] std::io::Error),

    /// Database or model operation failed.
    #[error(transparent)]
    Models(#[from] ej_models::error::Error),

    /// Authentication operation failed.
    #[error(transparent)]
    Auth(#[from] ej_auth::error::Error),

    /// JSON serialization/deserialization failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// Failed to dispatch job to builder.
    #[error("Internal error dispatching job")]
    InternalErrorDispatchingJob,

    /// Invalid job type specified.
    #[error("Invalid Job Type")]
    InvalidJobType,

    /// No builders are currently available to process jobs.
    #[error("No builders available")]
    NoBuildersAvailable,

    /* Api Errors */
    /// API access is forbidden for the current user.
    #[error("API Forbidden")]
    ApiForbidden,

    /// Failed to create authentication token.
    #[error("Auth Token Creation")]
    AuthTokenCreation,

    /// Invalid credentials provided.
    #[error("Wrong Credentials")]
    WrongCredentials,

    /// Required credentials are missing.
    #[error("Missing Credentials")]
    MissingCredentials,

    /// Request context is missing.
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
