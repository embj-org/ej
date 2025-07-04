use crate::prelude::*;
use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;
use tracing::error;

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
            | Error::JWT(_)
            | Error::PasswordHash(_)
            | Error::CtxMissing
            | Error::Json(_)
            | Error::Toml(_)
            | Error::BuildError
            | Error::RunError
            | Error::ChannelSendError
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
