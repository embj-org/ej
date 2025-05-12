use std::collections::HashSet;

use chrono::{TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::jwt::{jwt_decode, jwt_encode};
use crate::auth::secret_hash::is_secret_valid;
use crate::ctx::ctx_client::CtxClient;
use crate::db::connection::DbConnection;
use crate::ej_client::api::{EjClientApi, EjClientLoginRequest};
use crate::ej_client::db::EjClient;
use crate::permission::Permission;
use crate::prelude::*;

use super::auth_body::AuthBody;

const ISS: &str = "EJ";

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthToken {
    pub sub: Uuid,
    pub iss: String,
    pub exp: i64,
    pub iat: i64,
    pub nbf: i64,
    pub jti: Uuid,

    pub permissions: HashSet<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_data: Option<CtxClient>,
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

impl AuthToken {
    pub fn new(
        client_id: &Uuid,
        permissions: HashSet<String>,
        token_duration: TimeDelta,
    ) -> Result<Self> {
        let expiration = chrono::Utc::now()
            .checked_add_signed(token_duration)
            .ok_or_else(|| Error::AuthTokenCreation)?;

        Ok(Self {
            sub: *client_id,
            exp: expiration.timestamp(),
            iat: Utc::now().timestamp(),
            nbf: Utc::now().timestamp(),
            iss: String::from(ISS),
            jti: Uuid::new_v4(),
            permissions,
            client_data: None,
        })
    }
}

pub fn authenticate(
    auth: &EjClientLoginRequest,
    connection: &DbConnection,
) -> Result<(EjClientApi, Vec<Permission>)> {
    if auth.secret.is_empty() {
        return Err(Error::MissingCredentials);
    }
    let client = EjClient::fetch_by_name(&auth.name, connection)?;
    let is_valid = is_secret_valid(&auth.secret, &client.hash)?;
    if !is_valid {
        return Err(Error::WrongCredentials);
    }
    let permissions = client.fetch_permissions(connection)?;
    Ok((client.into(), permissions))
}

pub fn encode_token(token: &AuthToken) -> Result<AuthBody> {
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
