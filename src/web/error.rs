use crate::prelude::*;
use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            // Auth-related errors
            Error::AuthTokenMissing => (StatusCode::UNAUTHORIZED, "Authentication required"),
            Error::AuthTokenExpired => (StatusCode::UNAUTHORIZED, "Authentication token expired"),
            Error::AuthInvalidToken => (StatusCode::UNAUTHORIZED, "Invalid authentication token"),
            Error::WrongCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            Error::MissingCredentials => (StatusCode::UNAUTHORIZED, "Missing credentials"),

            // Permission-related errors
            Error::ApiForbidden => (StatusCode::FORBIDDEN, "Access forbidden"),

            // Internal errors - hide details
            Error::AuthTokenCreation
            | Error::Generic(_)
            | Error::IO(_)
            | Error::JWT(_)
            | Error::PasswordHash(_)
            | Error::R2D2(_)
            | Error::Diesel(_)
            | Error::CtxMissing => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
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
