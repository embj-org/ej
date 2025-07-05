//! Trait for job result serialization and persistence.

use crate::prelude::*;
use ej_models::db::connection::DbConnection;
use uuid::Uuid;

/// Trait for objects that represent job execution results.
pub trait EjJobResult {
    /// Saves the job result to the database.
    fn save(self, connection: &DbConnection) -> Result<()>;

    /// Returns the job ID this result belongs to.
    fn job_id(&self) -> Uuid;

    /// Returns the builder ID that produced this result.
    fn builder_id(&self) -> Uuid;
}
