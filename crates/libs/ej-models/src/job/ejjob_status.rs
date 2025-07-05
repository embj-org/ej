//! Job status definitions for tracking job execution state.

use crate::prelude::*;
use crate::{db::connection::DbConnection, schema::ejjobstatus::dsl::*};
use diesel::associations::HasTable;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A job status that tracks the execution state of jobs.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, PartialEq, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::ejjobstatus)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EjJobStatus {
    /// The unique status ID.
    pub id: i32,
    /// The status name.
    pub status: String,
}

/// Data for creating a new job status.
#[derive(Insertable, PartialEq, Debug, Clone, Deserialize)]
#[diesel(table_name = crate::schema::ejjobstatus)]
pub struct EjJobStatusCreate {
    /// The status ID.
    pub id: i32,
    /// The status name.
    pub status: String,
}

impl EjJobStatus {
    /// Returns the ID for jobs that haven't started.
    pub fn not_started() -> i32 {
        0
    }

    /// Returns the ID for currently running jobs.
    pub fn running() -> i32 {
        1
    }

    /// Returns the ID for successfully completed jobs.
    pub fn success() -> i32 {
        2
    }

    /// Returns the ID for failed jobs.
    pub fn failed() -> i32 {
        3
    }

    /// Returns the ID for cancelled jobs.
    pub fn cancelled() -> i32 {
        4
    }
}

impl EjJobStatusCreate {
    /// Saves the job status to the database.
    pub fn save(self, connection: &DbConnection) -> Result<EjJobStatus> {
        let conn = &mut connection.pool.get()?;
        Ok(diesel::insert_into(ejjobstatus)
            .values(&self)
            .returning(EjJobStatus::as_returning())
            .get_result(conn)?
            .into())
    }
}

impl EjJobStatus {
    /// Fetches a job status by its ID.
    pub fn fetch_by_id(target: i32, connection: &DbConnection) -> Result<Self> {
        let conn = &mut connection.pool.get()?;
        let job_status: EjJobStatus = EjJobStatus::by_id(target)
            .select(EjJobStatus::as_select())
            .get_result(conn)?;
        Ok(job_status.into())
    }

    /// Fetches a job status by its name.
    pub fn fetch_by_status(target: &str, connection: &DbConnection) -> Result<Self> {
        let conn = &mut connection.pool.get()?;
        let job_status: EjJobStatus = EjJobStatus::by_status(target)
            .select(EjJobStatus::as_select())
            .get_result(conn)?;
        Ok(job_status.into())
    }

    /// Fetches all job statuses from the database.
    pub fn fetch_all(connection: &DbConnection) -> Result<Vec<Self>> {
        let conn = &mut connection.pool.get()?;
        Ok(EjJobStatus::table()
            .select(EjJobStatus::as_select())
            .load(conn)?)
    }

    /// Returns a query filtered by status ID.
    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_id(target: i32) -> _ {
        crate::schema::ejjobstatus::dsl::ejjobstatus.filter(id.eq(target))
    }

    /// Returns a query filtered by status name.
    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_status(target: &str) -> _ {
        crate::schema::ejjobstatus::dsl::ejjobstatus.filter(status.eq(target))
    }
}
