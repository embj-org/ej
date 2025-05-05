use std::collections::HashSet;

use chrono::{TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;

use crate::auth::jwt::{jwt_decode, jwt_encode};
use crate::auth::secret_hash::is_secret_valid;
use crate::db::connection::DbConnection;
use crate::ej_client::api::{EjClientApi, EjClientLogin};
use crate::ej_client::db::EjClient;
use crate::permission::Permission;
use crate::prelude::*;

use super::ctx::AUTH_TOKEN_COOKIE;
use super::ctx::CtxClient;

const TOKEN_EXPIRATION_TIME: TimeDelta = TimeDelta::hours(12);
const ISS: &str = "EJ";
pub const TOKEN_TYPE: &'static str = "Bearer";

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthBody {
    pub access_token: String,
    pub token_type: String,
}

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

impl AuthBody {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: String::from(TOKEN_TYPE),
        }
    }
}

impl AuthToken {
    pub fn new(
        client: &EjClientApi,
        client_permissions: Vec<Permission>,
        token_duration: TimeDelta,
    ) -> Result<Self> {
        let expiration = chrono::Utc::now()
            .checked_add_signed(token_duration)
            .ok_or_else(|| Error::AuthTokenCreation)?;

        Ok(Self {
            sub: client.id,
            exp: expiration.timestamp(),
            iat: Utc::now().timestamp(),
            nbf: Utc::now().timestamp(),
            iss: String::from(ISS),
            jti: Uuid::new_v4(),
            permissions: client_permissions.iter().map(|p| p.id.clone()).collect(),
            client_data: None,
        })
    }
}

pub fn authenticate(
    auth: &EjClientLogin,
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

pub fn authenticate_and_generate_token(
    auth: &EjClientLogin,
    connection: &DbConnection,
    cookies: &Cookies,
) -> Result<AuthBody> {
    let (client, permissions) = authenticate(auth, connection)?;
    let token = generate_token(&client, permissions)?;
    cookies.add(Cookie::new(AUTH_TOKEN_COOKIE, token.access_token.clone()));

    Ok(token)
}

pub fn generate_token(client: &EjClientApi, permissions: Vec<Permission>) -> Result<AuthBody> {
    let claims = AuthToken::new(client, permissions, TOKEN_EXPIRATION_TIME)?;
    encode_token(&claims)
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
