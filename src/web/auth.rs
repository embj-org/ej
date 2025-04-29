use chrono::TimeDelta;
use serde::{Deserialize, Serialize};

use crate::ej_client::EjClient;
use crate::prelude::*;

use super::jwt::{jwt_decode, jwt_encode};

const COMPANY_NAME: &str = "EJ";
pub const TOKEN_TYPE: &str = "Bearer";

#[derive(Debug, Deserialize)]
pub struct AuthPayload {
    pub username: String,
    pub password: String,
}
#[derive(Debug, Serialize)]
pub struct AuthBody {
    pub access_token: String,
    pub token_type: &'static str,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthToken {
    pub sub: EjClient,
    pub company: String,
    pub exp: i64,
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum AuthError {
    #[error("Invalid Token")]
    InvalidToken,
    #[error("Token Missing")]
    TokenMissing,
    #[error("Token Expired")]
    TokenExpired,
    #[error(transparent)]
    TokenCreation(#[from] jsonwebtoken::errors::Error),
}

impl From<AuthError> for Error {
    fn from(value: AuthError) -> Self {
        match value {
            AuthError::TokenCreation(jwt_error) => Self::JWT(jwt_error),
            AuthError::InvalidToken => Self::AuthInvalidToken,
            AuthError::TokenMissing => Self::AuthTokenMissing,
            AuthError::TokenExpired => Self::AuthTokenExpired,
        }
    }
}

impl AuthBody {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: TOKEN_TYPE,
        }
    }
}

impl AuthToken {
    pub fn new(client: EjClient, token_duration: TimeDelta) -> Option<Self> {
        let expiration = chrono::Utc::now().checked_add_signed(token_duration)?;
        Some(Self {
            sub: client,
            company: String::from(COMPANY_NAME),
            exp: expiration.timestamp(),
        })
    }
}

pub fn encode_token(token: &AuthToken) -> std::result::Result<AuthBody, AuthError> {
    let token = jwt_encode(&token).map_err(|err| {
        log::error!("Failed to encode JWT {err}");
        AuthError::TokenCreation(err)
    })?;

    Ok(AuthBody::new(token))
}
pub fn decode_token(token: &str) -> std::result::Result<AuthToken, AuthError> {
    Ok(jwt_decode::<AuthToken>(token)
        .map_err(|err| {
            log::error!("Failed to decode jwt token {err}");
            AuthError::InvalidToken
        })?
        .claims)
}
