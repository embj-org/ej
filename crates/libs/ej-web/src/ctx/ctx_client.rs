//! Client context for authenticated web requests.

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

/// Client context containing authenticated client information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtxClient {
    /// The unique client ID.
    pub id: Uuid,
}

const BUILDER_TOKEN_EXPIRATION_TIME: TimeDelta = TimeDelta::days(365);
const CLIENT_TOKEN_EXPIRATION_TIME: TimeDelta = TimeDelta::hours(12);
const BUILDER_PERMISSIONS: [&'static str; 1] = ["builder"];

impl CtxClient {
    /// Creates a new builder for this client.
    ///
    /// Generates a builder instance with appropriate permissions and authentication token.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ej_web::ctx::ctx_client::CtxClient;
    /// use uuid::Uuid;
    /// # use ej_models::db::connection::DbConnection;
    ///
    /// # async fn example(mut conn: DbConnection) -> Result<(), Box<dyn std::error::Error>> {
    /// let client = CtxClient {
    ///     id: Uuid::new_v4(),
    /// };
    ///
    /// let builder = client.create_builder(&mut conn)?;
    /// println!("Created builder with ID: {} and token", builder.id);
    /// # Ok(())
    /// # }
    /// ```
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

    /// Connects this client as a builder with WebSocket communication.
    ///
    /// Creates an `EjConnectedBuilder` that can receive messages via WebSocket.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ej_web::ctx::ctx_client::CtxClient;
    /// use ej_dispatcher_sdk::ejws_message::EjWsServerMessage;
    /// use tokio::sync::mpsc;
    /// use std::net::SocketAddr;
    /// use uuid::Uuid;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = CtxClient {
    ///     id: Uuid::new_v4(),
    /// };
    ///
    /// let (tx, _rx) = mpsc::channel::<EjWsServerMessage>(100);
    /// let addr: SocketAddr = "127.0.0.1:8080".parse()?;
    ///
    /// let connected_builder = client.connect(tx, addr);
    /// println!("Builder connected from: {}", connected_builder.addr);
    /// # Ok(())
    /// # }
    /// ```
    pub fn connect(self, tx: Sender<EjWsServerMessage>, addr: SocketAddr) -> EjConnectedBuilder {
        EjConnectedBuilder {
            builder: self,
            tx,
            addr,
            connection_id: Uuid::new_v4(),
        }
    }
}

/// Generates an authentication token for a client with specified permissions.
///
/// Creates a JWT token that can be used for authenticating API requests.
///
/// # Examples
///
/// ```rust
/// use ej_web::ctx::ctx_client::generate_token;
/// use ej_dispatcher_sdk::ejclient::EjClientApi;
/// use ej_models::auth::permission::Permission;
/// use uuid::Uuid;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = EjClientApi {
///     id: Uuid::new_v4(),
///     name: "example-client".to_string(),
/// };
///
/// let permissions = vec![
///     Permission::new("read".to_string()),
///     Permission::new("write".to_string()),
/// ];
///
/// let auth_body = generate_token(&client, permissions)?;
/// println!("Generated token: {}", auth_body.access_token);
/// # Ok(())
/// # }
/// ```
pub fn generate_token(client: &EjClientApi, permissions: Vec<Permission>) -> Result<AuthBody> {
    let permissions: HashSet<String> = permissions.into_iter().map(|p| p.id).collect();
    let claims = AuthToken::new_client(&client.id, permissions, CLIENT_TOKEN_EXPIRATION_TIME)?;
    encode_token(&claims)
}
