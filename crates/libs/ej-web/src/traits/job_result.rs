use crate::prelude::*;
use ej_models::db::connection::DbConnection;
use uuid::Uuid;

pub trait EjJobResult {
    fn save(self, connection: &DbConnection) -> Result<()>;
    fn job_id(&self) -> Uuid;
    fn builder_id(&self) -> Uuid;
}
