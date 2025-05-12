use std::collections::HashSet;

use crate::{
    auth::auth::{AuthToken, encode_token},
    prelude::*,
};
use chrono::TimeDelta;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::db::EjBuilder;

#[derive(Debug, Serialize, Deserialize)]
pub struct EjBuilderApi {
    pub id: Uuid,
    pub token: String,
}

impl EjBuilderApi {
    const TOKEN_EXPIRATION_TIME: TimeDelta = TimeDelta::days(365);
    const PERMISSIONS: [&'static str; 1] = ["builder"];
}

impl TryFrom<EjBuilder> for EjBuilderApi {
    type Error = Error;

    fn try_from(value: EjBuilder) -> std::result::Result<Self, Self::Error> {
        let permissions: HashSet<String> = Self::PERMISSIONS
            .into_iter()
            .map(|p| String::from(p))
            .collect();

        let claims = AuthToken::new(&value.ejclient_id, permissions, Self::TOKEN_EXPIRATION_TIME)?;
        let token = encode_token(&claims)?;
        Ok(Self {
            id: value.id,
            token: token.access_token,
        })
    }
}
