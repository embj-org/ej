use std::{collections::HashSet, fmt};

use crate::{
    crypto::{
        auth::{AuthToken, encode_token},
        auth_body::AuthBody,
        secret_hash::generate_secret_hash,
    },
    db::connection::DbConnection,
    permission::Permission,
    prelude::*,
};
use chrono::TimeDelta;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::db::{EjClient, EjClientCreate};

#[derive(Debug, Serialize, Deserialize)]
pub struct EjClientApi {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EjClientPost {
    pub name: String,
    pub secret: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EjClientLoginRequest {
    pub name: String,
    pub secret: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EjClientLogin {
    pub access_token: String,
    pub token_type: String,
}

impl From<AuthBody> for EjClientLogin {
    fn from(value: AuthBody) -> Self {
        Self {
            access_token: value.access_token,
            token_type: value.token_type,
        }
    }
}

impl From<EjClient> for EjClientApi {
    fn from(value: EjClient) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}

impl TryFrom<EjClientPost> for EjClientCreate {
    type Error = Error;

    fn try_from(value: EjClientPost) -> Result<Self> {
        let hash = generate_secret_hash(&value.secret)?;
        Ok(Self {
            name: value.name,
            hash,
            hash_version: 1,
        })
    }
}

impl EjClientApi {
    const TOKEN_EXPIRATION_TIME: TimeDelta = TimeDelta::hours(12);

    pub fn generate_token(&self, permissions: Vec<Permission>) -> Result<AuthBody> {
        let permissions: HashSet<String> = permissions.into_iter().map(|p| p.id).collect();
        let claims = AuthToken::new_client(&self.id, permissions, Self::TOKEN_EXPIRATION_TIME)?;
        encode_token(&claims)
    }
}

impl EjClientPost {
    pub fn persist(self, connection: &DbConnection) -> Result<EjClientApi> {
        let model: EjClientCreate = self.try_into()?;
        Ok(model.save(connection)?.into())
    }
}
impl EjClientLoginRequest {
    pub fn new(name: impl Into<String>, secret: impl Into<String>) -> Self {
        let name = name.into();
        let secret = secret.into();
        Self { name, secret }
    }
}
impl fmt::Display for EjClientApi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Client '{}' (ID: {})", self.name, self.id)
    }
}
