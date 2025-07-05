//! Job log management for tracking execution output.

use crate::config::ejboard_config::EjBoardConfigDb;
use crate::job::ejjob::EjJobDb;
use crate::prelude::*;
use crate::{db::connection::DbConnection, schema::ejjoblog::dsl::*};
use chrono::{DateTime, Utc};
use diesel::associations::HasTable;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A log entry for a job execution.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, PartialEq, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::ejjoblog)]
#[diesel(belongs_to(EjJob))]
#[diesel(belongs_to(EjBoardConfigDb))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EjJobLog {
    /// Unique log entry ID.
    pub id: Uuid,
    /// The job this log belongs to.
    pub ejjob_id: Uuid,
    /// The board config this log is associated with.
    pub ejboard_config_id: Uuid,
    /// The log content.
    pub log: String,
    /// When this log entry was created.
    pub created_at: DateTime<Utc>,
    /// When this log entry was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Data for creating a new job log entry.
#[derive(Insertable, PartialEq, Debug, Clone, Deserialize)]
#[diesel(table_name = crate::schema::ejjoblog)]
pub struct EjJobLogCreate {
    /// The job ID this log belongs to.
    pub ejjob_id: Uuid,
    /// The board config ID this log is associated with.
    pub ejboard_config_id: Uuid,
    /// The log content.
    pub log: String,
}

impl EjJobLogCreate {
    /// Saves the job log to the database.
    pub fn save(self, connection: &DbConnection) -> Result<EjJobLog> {
        let conn = &mut connection.pool.get()?;
        Ok(diesel::insert_into(ejjoblog)
            .values(&self)
            .returning(EjJobLog::as_returning())
            .get_result(conn)?
            .into())
    }
}

impl EjJobLog {
    /// Fetches a job log by its ID.
    pub fn fetch_by_id(target: &Uuid, connection: &DbConnection) -> Result<Self> {
        let conn = &mut connection.pool.get()?;
        let job_log: EjJobLog = EjJobLog::by_id(target)
            .select(EjJobLog::as_select())
            .get_result(conn)?;
        Ok(job_log.into())
    }

    /// Fetches all logs for a specific job.
    pub fn fetch_by_job_id(target: &Uuid, connection: &DbConnection) -> Result<Vec<Self>> {
        let conn = &mut connection.pool.get()?;
        Ok(EjJobLog::by_job_id(target)
            .select(EjJobLog::as_select())
            .load(conn)?)
    }

    /// Fetches job logs with their associated board config.
    pub fn fetch_with_board_config_by_job_id(
        target: &Uuid,
        connection: &DbConnection,
    ) -> Result<Vec<(EjJobLog, EjBoardConfigDb)>> {
        let conn = &mut connection.pool.get()?;

        let results = EjJobLog::by_job_id(target)
            .inner_join(crate::schema::ejboard_config::table)
            .select((EjJobLog::as_select(), EjBoardConfigDb::as_select()))
            .load::<(EjJobLog, EjBoardConfigDb)>(conn)?;

        Ok(results)
    }

    /// Fetches all logs for a specific board config.
    pub fn fetch_by_board_config_id(target: &Uuid, connection: &DbConnection) -> Result<Vec<Self>> {
        let conn = &mut connection.pool.get()?;
        Ok(EjJobLog::by_board_config_id(target)
            .select(EjJobLog::as_select())
            .load(conn)?)
    }

    /// Fetches logs for a specific job and board config combination.
    pub fn fetch_by_job_and_board(
        job_id: &Uuid,
        board_config_id: &Uuid,
        connection: &DbConnection,
    ) -> Result<Vec<Self>> {
        let conn = &mut connection.pool.get()?;
        Ok(EjJobLog::by_job_and_board(job_id, board_config_id)
            .select(EjJobLog::as_select())
            .load(conn)?)
    }

    /// Fetches the job associated with this log.
    pub fn fetch_job(&self, connection: &DbConnection) -> Result<EjJobDb> {
        EjJobDb::fetch_by_id(&self.ejjob_id, connection)
    }

    /// Fetches all job logs from the database.
    pub fn fetch_all(connection: &DbConnection) -> Result<Vec<Self>> {
        let conn = &mut connection.pool.get()?;
        Ok(EjJobLog::table().select(EjJobLog::as_select()).load(conn)?)
    }

    /// Returns a query filtered by log ID.
    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_id(target: &Uuid) -> _ {
        crate::schema::ejjoblog::dsl::ejjoblog.filter(id.eq(target))
    }

    /// Returns a query filtered by job ID.
    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_job_id(target: &Uuid) -> _ {
        crate::schema::ejjoblog::dsl::ejjoblog.filter(ejjob_id.eq(target))
    }

    /// Returns a query filtered by board config ID.
    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_board_config_id(target: &Uuid) -> _ {
        crate::schema::ejjoblog::dsl::ejjoblog.filter(ejboard_config_id.eq(target))
    }

    /// Returns a query filtered by both job and board config ID.
    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_job_and_board<'a>(job_id: &'a Uuid, board_config_id: &'a Uuid) -> _ {
        crate::schema::ejjoblog::dsl::ejjoblog
            .filter(ejjob_id.eq(job_id))
            .filter(ejboard_config_id.eq(board_config_id))
    }
}
