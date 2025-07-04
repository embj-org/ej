use crate::{
    ej_client::api::{EjClientApi, EjClientLoginRequest},
    prelude::*,
    web::ctx::{CtxWho, ctx_client::CtxClient},
};
use std::collections::HashSet;

use chrono::{TimeDelta, Utc};
use lib_auth::{
    ISS,
    auth_body::AuthBody,
    jwt::{jwt_decode, jwt_encode},
    secret_hash::is_secret_valid,
};
use lib_models::{
    auth::permission::Permission, client::ejclient::EjClient, db::connection::DbConnection,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

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
    pub who: CtxWho,
}

impl AuthToken {
    pub fn new_client(
        id: &Uuid,
        permissions: HashSet<String>,
        token_duration: TimeDelta,
    ) -> Result<Self> {
        let expiration = chrono::Utc::now()
            .checked_add_signed(token_duration)
            .ok_or_else(|| Error::AuthTokenCreation)?;

        Ok(Self {
            sub: *id,
            exp: expiration.timestamp(),
            iat: Utc::now().timestamp(),
            nbf: Utc::now().timestamp(),
            iss: String::from(ISS),
            jti: Uuid::new_v4(),
            permissions,
            client_data: None,
            who: CtxWho::Client,
        })
    }
    pub fn new_builder(
        id: &Uuid,
        permissions: HashSet<String>,
        token_duration: TimeDelta,
    ) -> Result<Self> {
        let expiration = chrono::Utc::now()
            .checked_add_signed(token_duration)
            .ok_or_else(|| Error::AuthTokenCreation)?;

        Ok(Self {
            sub: *id,
            exp: expiration.timestamp(),
            iat: Utc::now().timestamp(),
            nbf: Utc::now().timestamp(),
            iss: String::from(ISS),
            jti: Uuid::new_v4(),
            permissions,
            client_data: None,
            who: CtxWho::Builder,
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
        error!("Failed to encode JWT {err}");
        err
    })?;

    Ok(AuthBody::new(token))
}
pub fn decode_token(token: &str) -> Result<AuthToken> {
    Ok(jwt_decode::<AuthToken>(token)
        .map_err(|err| {
            log::error!("Failed to decode jwt token {err}");
            err
        })?
        .claims)
}
