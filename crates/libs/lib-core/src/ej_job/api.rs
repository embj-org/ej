use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{db::connection::DbConnection, ej_job::db::EjJobCreate, prelude::*};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EjJob {
    pub commit_hash: String,
    pub remote_url: String,
    pub remote_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EjDeployableJob {
    pub id: Uuid,
    pub commit_hash: String,
    pub remote_url: String,
    pub remote_token: Option<String>,
}

impl EjJob {
    pub fn create(self, connection: &mut DbConnection) -> Result<EjDeployableJob> {
        let job = EjJobCreate {
            commit_hash: self.commit_hash,
            remote_url: self.remote_url,
        };
        let job = job.save(connection)?;

        Ok(EjDeployableJob {
            id: job.id,
            commit_hash: job.commit_hash,
            remote_url: job.remote_url,
            remote_token: self.remote_token,
        })
    }
}
