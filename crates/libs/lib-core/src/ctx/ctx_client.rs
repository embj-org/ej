use crate::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    db::connection::DbConnection,
    ej_builder::{api::EjBuilderApi, db::EjBuilderCreate},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtxClient {
    pub id: Uuid,
}

impl CtxClient {
    pub fn create_builder(&self, conn: &mut DbConnection) -> Result<EjBuilderApi> {
        Ok(EjBuilderCreate::new(self.id).create(conn)?.try_into()?)
    }
}
