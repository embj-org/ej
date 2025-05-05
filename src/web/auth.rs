use chrono::TimeDelta;
use serde::{Deserialize, Serialize};
use tower_cookies::{Cookie, Cookies};

use crate::auth::jwt::{jwt_decode, jwt_encode};
use crate::auth::secret_hash::is_secret_valid;
use crate::db::connection::DbConnection;
use crate::ej_client::api::{EjClientApi, EjClientLogin};
use crate::ej_client::db::EjClient;
use crate::prelude::*;

use super::ctx::AUTH_TOKEN_COOKIE;
use super::ctx::CtxClient;

const TOKEN_EXPIRATION_TIME: TimeDelta = TimeDelta::hours(12);
const COMPANY_NAME: &str = "EJ";
pub const TOKEN_TYPE: &'static str = "Bearer";

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthBody {
    pub access_token: String,
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthToken {
    pub sub: CtxClient,
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
            token_type: String::from(TOKEN_TYPE),
        }
    }
}

impl AuthToken {
    pub fn new(client: &EjClientApi, token_duration: TimeDelta) -> Result<Self> {
        let expiration = chrono::Utc::now()
            .checked_add_signed(token_duration)
            .ok_or_else(|| Error::AuthTokenCreation)?;

        let sub = CtxClient {
            client_id: client.id,
            client_name: client.name.clone(),
        };
        Ok(Self {
            sub,
            company: String::from(COMPANY_NAME),
            exp: expiration.timestamp(),
        })
    }
}

pub fn authenticate(auth: &EjClientLogin, connection: &DbConnection) -> Result<EjClientApi> {
    if auth.secret.is_empty() {
        return Err(Error::MissingCredentials);
    }
    let client = EjClient::fetch_by_name(&auth.name, connection)?;
    let is_valid = is_secret_valid(&auth.secret, &client.hash)?;
    if !is_valid {
        return Err(Error::WrongCredentials);
    }
    Ok(client.into())
}

pub fn authenticate_and_generate_token(
    auth: &EjClientLogin,
    connection: &DbConnection,
    cookies: &Cookies,
) -> Result<AuthBody> {
    let user = authenticate(auth, connection)?;
    let token = generate_token(&user)?;
    cookies.add(Cookie::new(AUTH_TOKEN_COOKIE, token.access_token.clone()));

    Ok(token)
}

pub fn generate_token(client: &EjClientApi) -> Result<AuthBody> {
    let claims = AuthToken::new(client, TOKEN_EXPIRATION_TIME)?;
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
