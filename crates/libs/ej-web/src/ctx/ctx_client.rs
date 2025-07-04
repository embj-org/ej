use std::collections::HashSet;
use std::net::SocketAddr;

use chrono::TimeDelta;
use ej_auth::auth_body::AuthBody;
use ej_dispatcher_sdk::ejbuilder::EjBuilderApi;
use ej_dispatcher_sdk::ejclient::EjClientApi;
use ej_dispatcher_sdk::ejws_message::EjWsServerMessage;
use ej_models::auth::permission::Permission;
use ej_models::{builder::ejbuilder::EjBuilderCreate, db::connection::DbConnection};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::auth_token::{AuthToken, encode_token};
use crate::ejconnected_builder::EjConnectedBuilder;
use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtxClient {
    pub id: Uuid,
}

const BUILDER_TOKEN_EXPIRATION_TIME: TimeDelta = TimeDelta::days(365);
const CLIENT_TOKEN_EXPIRATION_TIME: TimeDelta = TimeDelta::hours(12);
const BUILDER_PERMISSIONS: [&'static str; 1] = ["builder"];

impl CtxClient {
    pub fn create_builder(&self, conn: &mut DbConnection) -> Result<EjBuilderApi> {
        let builder = EjBuilderCreate::new(self.id).create(conn)?;

        let permissions: HashSet<String> = BUILDER_PERMISSIONS
            .into_iter()
            .map(|p| String::from(p))
            .collect();

        let claims =
            AuthToken::new_builder(&builder.id, permissions, BUILDER_TOKEN_EXPIRATION_TIME)?;
        let token = encode_token(&claims)?;
        Ok(EjBuilderApi {
            id: builder.id,
            token: token.access_token,
        })
    }

    pub fn connect(self, tx: Sender<EjWsServerMessage>, addr: SocketAddr) -> EjConnectedBuilder {
        EjConnectedBuilder {
            builder: self,
            tx,
            addr,
        }
    }
}

pub fn generate_token(client: &EjClientApi, permissions: Vec<Permission>) -> Result<AuthBody> {
    let permissions: HashSet<String> = permissions.into_iter().map(|p| p.id).collect();
    let claims = AuthToken::new_client(&client.id, permissions, CLIENT_TOKEN_EXPIRATION_TIME)?;
    encode_token(&claims)
}
