use crate::{auth::secret_hash::generate_secret_hash, db::connection::DbConnection, prelude::*};
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
pub struct EjClientLogin {
    pub name: String,
    pub secret: String,
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

impl EjClientPost {
    pub fn persist(self, connection: &DbConnection) -> Result<EjClientApi> {
        let model: EjClientCreate = self.try_into()?;
        Ok(model.save(connection)?.into())
    }
}
