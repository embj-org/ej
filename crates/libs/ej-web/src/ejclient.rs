//! Client management utilities for web handlers.

use ej_auth::{auth_body::AuthBody, secret_hash::generate_secret_hash};
use ej_dispatcher_sdk::ejclient::{EjClientApi, EjClientLogin, EjClientPost};
use ej_models::{client::ejclient::EjClientCreate, db::connection::DbConnection};

use crate::prelude::*;

impl From<AuthBody> for W<EjClientLogin> {
    fn from(value: AuthBody) -> Self {
        Self(EjClientLogin {
            access_token: value.access_token,
            token_type: value.token_type,
        })
    }
}

/// Creates a new client from the provided payload.
///
/// # Examples
///
/// ```rust
/// use ej_web::ejclient::create_client;
/// use ej_dispatcher_sdk::ejclient::EjClientPost;
/// # use ej_models::db::connection::DbConnection;
///
/// # async fn example(connection: &DbConnection) -> Result<(), Box<dyn std::error::Error>> {
/// let payload = EjClientPost {
///     name: "example-client".to_string(),
///     secret: "secret123".to_string(),
/// };
/// let client = create_client(payload, connection)?;
/// # Ok(())
/// # }
/// ```
pub fn create_client(payload: EjClientPost, connection: &DbConnection) -> Result<EjClientApi> {
    let hash = generate_secret_hash(&payload.secret)?;
    let model = EjClientCreate {
        name: payload.name,
        hash,
        hash_version: 1,
    };
    let model = model.save(connection)?;

    let result = EjClientApi {
        id: model.id,
        name: model.name,
    };
    Ok(result)
}
