//! Authentication token management for web requests.

use std::collections::HashSet;

use crate::prelude::*;
use chrono::{TimeDelta, Utc};
use ej_auth::{
    ISS,
    auth_body::AuthBody,
    jwt::{jwt_decode, jwt_encode},
    secret_hash::is_secret_valid,
};
use ej_dispatcher_sdk::ejclient::{EjClientApi, EjClientLoginRequest};
use ej_models::{
    auth::permission::Permission, client::ejclient::EjClient, db::connection::DbConnection,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use crate::ctx::{CtxWho, ctx_client::CtxClient};

/// JWT authentication token containing user/builder identity and permissions.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthToken {
    /// Subject (user/builder ID).
    pub sub: Uuid,
    /// Issuer.
    pub iss: String,
    /// Expiration time.
    pub exp: i64,
    /// Issued at time.
    pub iat: i64,
    /// Not before time.
    pub nbf: i64,
    /// JWT ID.
    pub jti: Uuid,
    /// Granted permissions.
    pub permissions: HashSet<String>,
    /// Client data (for client tokens).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_data: Option<CtxClient>,
    pub who: CtxWho,
}

impl AuthToken {
    /// Creates a new client authentication token.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ej_web::auth_token::AuthToken;
    /// use std::collections::HashSet;
    /// use chrono::TimeDelta;
    /// use uuid::Uuid;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client_id = Uuid::new_v4();
    /// let mut permissions = HashSet::new();
    /// permissions.insert("read".to_string());
    /// permissions.insert("write".to_string());
    ///
    /// let token = AuthToken::new_client(
    ///     &client_id,
    ///     permissions,
    ///     TimeDelta::hours(12)
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
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

    /// Creates a new builder authentication token.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ej_web::auth_token::AuthToken;
    /// use std::collections::HashSet;
    /// use chrono::TimeDelta;
    /// use uuid::Uuid;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let builder_id = Uuid::new_v4();
    /// let mut permissions = HashSet::new();
    /// permissions.insert("builder".to_string());
    ///
    /// let token = AuthToken::new_builder(
    ///     &builder_id,
    ///     permissions,
    ///     TimeDelta::days(365)
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
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

/// Authenticates a client using login credentials.
///
/// Validates the provided credentials against the database and returns
/// the client information along with their permissions if successful.
///
/// # Examples
///
/// ```rust
/// use ej_web::auth_token::authenticate;
/// use ej_dispatcher_sdk::ejclient::EjClientLoginRequest;
/// # use ej_models::db::connection::DbConnection;
///
/// # async fn example(connection: &DbConnection) -> Result<(), Box<dyn std::error::Error>> {
/// let login_request = EjClientLoginRequest {
///     name: "example-client".to_string(),
///     secret: "client-secret".to_string(),
/// };
///
/// let (client, permissions) = authenticate(&login_request, connection)?;
/// println!("Authenticated client: {} with {} permissions", client.name, permissions.len());
/// # Ok(())
/// # }
/// ```
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
    Ok((
        EjClientApi {
            id: client.id,
            name: client.name,
        },
        permissions,
    ))
}

/// Encodes an authentication token into a JWT string.
///
/// # Examples
///
/// ```rust
/// use ej_web::auth_token::{AuthToken, encode_token};
/// use std::collections::HashSet;
/// use chrono::TimeDelta;
/// use uuid::Uuid;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client_id = Uuid::new_v4();
/// let permissions = HashSet::new();
///
/// let token = AuthToken::new_client(&client_id, permissions, TimeDelta::hours(12))?;
/// let encoded = encode_token(&token)?;
///
/// println!("Encoded token: {}", encoded.access_token);
/// # Ok(())
/// # }
/// ```
pub fn encode_token(token: &AuthToken) -> Result<AuthBody> {
    let token = jwt_encode(&token).map_err(|err| {
        error!("Failed to encode JWT {err}");
        err
    })?;

    Ok(AuthBody::new(token))
}

/// Decodes a JWT string back into an authentication token.
///
/// # Examples
///
/// ```rust
/// use ej_web::auth_token::{AuthToken, encode_token, decode_token};
/// use std::collections::HashSet;
/// use chrono::TimeDelta;
/// use uuid::Uuid;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client_id = Uuid::new_v4();
/// let permissions = HashSet::new();
///
/// // Create and encode a token
/// let original_token = AuthToken::new_client(&client_id, permissions, TimeDelta::hours(12))?;
/// let encoded = encode_token(&original_token)?;
///
/// // Decode it back
/// let decoded_token = decode_token(&encoded.access_token)?;
/// assert_eq!(original_token.sub, decoded_token.sub);
/// # Ok(())
/// # }
/// ```
pub fn decode_token(token: &str) -> Result<AuthToken> {
    Ok(jwt_decode::<AuthToken>(token)
        .map_err(|err| {
            log::error!("Failed to decode jwt token {err}");
            err
        })?
        .claims)
}
