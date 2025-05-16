use serde::{Deserialize, Serialize};

use crate::{db::connection::DbConnection, prelude::*};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EjJob {
    pub commit_hash: String,
    pub remote_url: String,
    pub remote_token: Option<String>,
}

impl EjJob {
    pub fn create(self, connection: &mut DbConnection) -> Result<Self> {
        Ok(self)
    }
}
