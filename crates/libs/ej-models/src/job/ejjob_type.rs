//! Job type definitions for categorizing different kinds of jobs.

use crate::prelude::*;
use crate::{db::connection::DbConnection, schema::ejjobtype::dsl::*};
use diesel::associations::HasTable;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A job type that categorizes different kinds of jobs.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, PartialEq, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::ejjobtype)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EjJobTypeDb {
    /// The unique job type ID.
    pub id: i32,
    /// The job type name.
    pub job_type: String,
}

/// Data for creating a new job type.
#[derive(Insertable, PartialEq, Debug, Clone, Deserialize)]
#[diesel(table_name = crate::schema::ejjobtype)]
pub struct EjJobTypeCreate {
    /// The job type ID.
    pub id: i32,
    /// The job type name.
    pub job_type: String,
}

impl EjJobTypeDb {
    /// Returns the ID for build jobs.
    pub fn build() -> i32 {
        0
    }

    /// Returns the ID for run jobs.
    pub fn run() -> i32 {
        1
    }
}

impl EjJobTypeCreate {
    /// Saves the job type to the database.
    pub fn save(self, connection: &DbConnection) -> Result<EjJobTypeDb> {
        let conn = &mut connection.pool.get()?;
        Ok(diesel::insert_into(ejjobtype)
            .values(&self)
            .returning(EjJobTypeDb::as_returning())
            .get_result(conn)?
            .into())
    }
}

impl EjJobTypeDb {
    /// Fetches a job type by its ID.
    pub fn fetch_by_id(target: i32, connection: &DbConnection) -> Result<Self> {
        let conn = &mut connection.pool.get()?;
        let job_status: EjJobTypeDb = EjJobTypeDb::by_id(target)
            .select(EjJobTypeDb::as_select())
            .get_result(conn)?;
        Ok(job_status.into())
    }

    /// Fetches a job type by its name.
    pub fn fetch_by_status(target: &str, connection: &DbConnection) -> Result<Self> {
        let conn = &mut connection.pool.get()?;
        let job_status: EjJobTypeDb = EjJobTypeDb::by_status(target)
            .select(EjJobTypeDb::as_select())
            .get_result(conn)?;
        Ok(job_status.into())
    }

    /// Fetches all job types from the database.
    pub fn fetch_all(connection: &DbConnection) -> Result<Vec<Self>> {
        let conn = &mut connection.pool.get()?;
        Ok(EjJobTypeDb::table()
            .select(EjJobTypeDb::as_select())
            .load(conn)?)
    }

    /// Returns a query filtered by job type ID.
    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_id(target: i32) -> _ {
        crate::schema::ejjobtype::dsl::ejjobtype.filter(id.eq(target))
    }

    /// Returns a query filtered by job type name.
    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_status(target: &str) -> _ {
        crate::schema::ejjobtype::dsl::ejjobtype.filter(job_type.eq(target))
    }
}
