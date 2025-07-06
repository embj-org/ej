//! Job model for managing job execution in the ej system.

use crate::db::connection::DbConnection;
use crate::job::ejjob_type::EjJobTypeDb;
use crate::prelude::*;
use crate::schema::ejjob::dsl::*;
use crate::schema::ejjob::status;
use crate::{config::ejboard_config::EjBoardConfigDb, job::ejjob_status::EjJobStatus};
use chrono::{DateTime, Utc};
use diesel::associations::HasTable;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A job that can be executed by the ej system.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, PartialEq, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::ejjob)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EjJobDb {
    /// Unique job ID.
    pub id: Uuid,
    /// Git commit hash for the job.
    pub commit_hash: String,
    /// Git remote URL for the job.
    pub remote_url: String,
    /// The type of job (build, run, etc.).
    pub job_type: i32,
    /// Current status of the job.
    pub status: i32,
    /// When the job was dispatched for execution.
    pub dispatched_at: Option<DateTime<Utc>>,
    /// When the job finished execution.
    pub finished_at: Option<DateTime<Utc>>,
    /// When this job was created.
    pub created_at: DateTime<Utc>,
    /// When this job was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Data for creating a new job.
#[derive(Insertable, PartialEq, Debug, Clone, Deserialize)]
#[diesel(table_name = crate::schema::ejjob)]
pub struct EjJobCreate {
    /// Git commit hash for the job.
    pub commit_hash: String,
    /// Git remote URL for the job.
    pub remote_url: String,
    /// The type of job to create.
    pub job_type: i32,
}

impl EjJobCreate {
    /// Saves the job to the database.
    pub fn save(self, connection: &DbConnection) -> Result<EjJobDb> {
        let conn = &mut connection.pool.get()?;
        Ok(diesel::insert_into(ejjob)
            .values(&self)
            .returning(EjJobDb::as_returning())
            .get_result(conn)?
            .into())
    }
}

impl EjJobDb {
    pub fn fetch_by_id(target: &Uuid, connection: &DbConnection) -> Result<Self> {
        let conn = &mut connection.pool.get()?;
        let job: EjJobDb = EjJobDb::by_id(target)
            .select(EjJobDb::as_select())
            .get_result(conn)?;
        Ok(job.into())
    }

    pub fn fetch_by_commit_hash(target: &str, connection: &DbConnection) -> Result<Vec<Self>> {
        let conn = &mut connection.pool.get()?;
        Ok(EjJobDb::by_commit_hash(target)
            .select(EjJobDb::as_select())
            .load(conn)?)
    }

    pub fn fetch_status(&self, connection: &DbConnection) -> Result<EjJobStatus> {
        Ok(EjJobStatus::fetch_by_id(self.status, connection)?)
    }

    pub fn fetch_type(&self, connection: &DbConnection) -> Result<EjJobTypeDb> {
        Ok(EjJobTypeDb::fetch_by_id(self.job_type, connection)?)
    }

    pub fn fetch_all(connection: &DbConnection) -> Result<Vec<Self>> {
        let conn = &mut connection.pool.get()?;
        Ok(EjJobDb::table().select(EjJobDb::as_select()).load(conn)?)
    }

    pub fn update_status(&self, new_status: i32, connection: &DbConnection) -> Result<Self> {
        let conn = &mut connection.pool.get()?;
        Ok(diesel::update(EjJobDb::by_id(&self.id))
            .set(status.eq(new_status))
            .returning(EjJobDb::as_returning())
            .get_result(conn)?
            .into())
    }
    pub fn success(&self) -> bool {
        self.status == EjJobStatus::success()
    }
}

impl EjJobDb {
    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_id(target: &Uuid) -> _ {
        crate::schema::ejjob::dsl::ejjob.filter(id.eq(target))
    }

    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_commit_hash(target: &str) -> _ {
        crate::schema::ejjob::dsl::ejjob.filter(commit_hash.eq(target))
    }

    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_remote_url(target: &str) -> _ {
        crate::schema::ejjob::dsl::ejjob.filter(remote_url.eq(target))
    }
}
