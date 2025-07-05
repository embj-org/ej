//! Job result management for storing execution outcomes.

use crate::config::ejboard_config::EjBoardConfigDb;
use crate::job::ejjob::EjJobDb;
use crate::prelude::*;
use crate::{db::connection::DbConnection, schema::ejjobresult::dsl::*};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A job result storing the outcome of job execution.
#[derive(Debug, Clone, Queryable, Selectable, PartialEq, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::ejjobresult)]
#[diesel(belongs_to(EjJob))]
#[diesel(belongs_to(EjBoardConfig))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EjJobResultDb {
    /// The job this result belongs to.
    pub ejjob_id: Uuid,
    /// The board config this result is associated with.
    pub ejboard_config_id: Uuid,
    /// The result content.
    pub result: String,
    /// When this result was created.
    pub created_at: DateTime<Utc>,
    /// When this result was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Data for creating a new job result.
#[derive(Insertable, PartialEq, Debug, Clone, Deserialize)]
#[diesel(table_name = crate::schema::ejjobresult)]
pub struct EjJobResultCreate {
    /// The job ID this result belongs to.
    pub ejjob_id: Uuid,
    /// The board config ID this result is associated with.
    pub ejboard_config_id: Uuid,
    /// The result content.
    pub result: String,
}

impl EjJobResultCreate {
    /// Saves the job result to the database.
    pub fn save(self, connection: &DbConnection) -> Result<EjJobResultDb> {
        let conn = &mut connection.pool.get()?;
        Ok(diesel::insert_into(ejjobresult)
            .values(&self)
            .returning(EjJobResultDb::as_returning())
            .get_result(conn)?
            .into())
    }
}

impl EjJobResultDb {
    /// Fetches a job result by its composite key (job_id, board_config_id).
    pub fn fetch_by_composite_key(
        job_id: &Uuid,
        board_config_id: &Uuid,
        connection: &DbConnection,
    ) -> Result<Self> {
        let conn = &mut connection.pool.get()?;
        let job_result: EjJobResultDb = EjJobResultDb::by_composite_key(job_id, board_config_id)
            .select(EjJobResultDb::as_select())
            .get_result(conn)?;
        Ok(job_result)
    }

    /// Fetches all results for a specific job.
    pub fn fetch_by_job_id(target: &Uuid, connection: &DbConnection) -> Result<Vec<Self>> {
        let conn = &mut connection.pool.get()?;
        Ok(EjJobResultDb::by_job_id(target)
            .select(EjJobResultDb::as_select())
            .load(conn)?)
    }

    /// Fetches all results for a specific board config.
    pub fn fetch_by_board_config_id(target: &Uuid, connection: &DbConnection) -> Result<Vec<Self>> {
        let conn = &mut connection.pool.get()?;
        Ok(EjJobResultDb::by_board_config_id(target)
            .select(EjJobResultDb::as_select())
            .load(conn)?)
    }

    pub fn fetch_with_board_config_by_job_id(
        target: &Uuid,
        connection: &DbConnection,
    ) -> Result<Vec<(EjJobResultDb, EjBoardConfigDb)>> {
        let conn = &mut connection.pool.get()?;

        let results = EjJobResultDb::by_job_id(target)
            .inner_join(crate::schema::ejboard_config::table)
            .select((EjJobResultDb::as_select(), EjBoardConfigDb::as_select()))
            .load::<(EjJobResultDb, EjBoardConfigDb)>(conn)?;

        Ok(results)
    }

    pub fn fetch_job(&self, connection: &DbConnection) -> Result<EjJobDb> {
        EjJobDb::fetch_by_id(&self.ejjob_id, connection)
    }

    pub fn update_result(&self, new_result: String, connection: &DbConnection) -> Result<Self> {
        let conn = &mut connection.pool.get()?;
        Ok(diesel::update(EjJobResultDb::by_composite_key(
            &self.ejjob_id,
            &self.ejboard_config_id,
        ))
        .set(result.eq(new_result))
        .returning(EjJobResultDb::as_returning())
        .get_result(conn)?
        .into())
    }

    pub fn delete(&self, connection: &DbConnection) -> Result<()> {
        let conn = &mut connection.pool.get()?;
        diesel::delete(EjJobResultDb::by_composite_key(
            &self.ejjob_id,
            &self.ejboard_config_id,
        ))
        .execute(conn)?;
        Ok(())
    }
}

impl EjJobResultDb {
    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_composite_key<'a>(job_id: &'a Uuid, board_config_id: &'a Uuid) -> _ {
        crate::schema::ejjobresult::dsl::ejjobresult
            .filter(ejjob_id.eq(job_id))
            .filter(ejboard_config_id.eq(board_config_id))
    }

    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_job_id(target: &Uuid) -> _ {
        crate::schema::ejjobresult::dsl::ejjobresult.filter(ejjob_id.eq(target))
    }

    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_board_config_id(target: &Uuid) -> _ {
        crate::schema::ejjobresult::dsl::ejjobresult.filter(ejboard_config_id.eq(target))
    }
}
