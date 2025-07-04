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
