use crate::ej_job::db::EjJob;
use crate::prelude::*;
use crate::{db::connection::DbConnection, schema::ejjoblog::dsl::*};
use chrono::{DateTime, Utc};
use diesel::associations::HasTable;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, PartialEq, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::ejjoblog)]
#[diesel(belongs_to(EjJob))]
#[diesel(belongs_to(EjBoardConfig))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EjJobLog {
    pub id: Uuid,
    pub ejjob_id: Uuid,
    pub ejboard_config_id: Uuid,
    pub log: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, PartialEq, Debug, Clone, Deserialize)]
#[diesel(table_name = crate::schema::ejjoblog)]
pub struct EjJobLogCreate {
    pub ejjob_id: Uuid,
    pub ejboard_config_id: Uuid,
    pub log: String,
}

impl EjJobLogCreate {
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
    pub fn fetch_by_id(target: &Uuid, connection: &DbConnection) -> Result<Self> {
        let conn = &mut connection.pool.get()?;
        let job_log: EjJobLog = EjJobLog::by_id(target)
            .select(EjJobLog::as_select())
            .get_result(conn)?;
        Ok(job_log.into())
    }

    pub fn fetch_by_job_id(target: &Uuid, connection: &DbConnection) -> Result<Vec<Self>> {
        let conn = &mut connection.pool.get()?;
        Ok(EjJobLog::by_job_id(target)
            .select(EjJobLog::as_select())
            .load(conn)?)
    }

    pub fn fetch_by_board_config_id(target: &Uuid, connection: &DbConnection) -> Result<Vec<Self>> {
        let conn = &mut connection.pool.get()?;
        Ok(EjJobLog::by_board_config_id(target)
            .select(EjJobLog::as_select())
            .load(conn)?)
    }

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

    pub fn fetch_job(&self, connection: &DbConnection) -> Result<EjJob> {
        EjJob::fetch_by_id(&self.ejjob_id, connection)
    }

    pub fn fetch_all(connection: &DbConnection) -> Result<Vec<Self>> {
        let conn = &mut connection.pool.get()?;
        Ok(EjJobLog::table().select(EjJobLog::as_select()).load(conn)?)
    }
}

impl EjJobLog {
    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_id(target: &Uuid) -> _ {
        crate::schema::ejjoblog::dsl::ejjoblog.filter(id.eq(target))
    }

    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_job_id(target: &Uuid) -> _ {
        crate::schema::ejjoblog::dsl::ejjoblog.filter(ejjob_id.eq(target))
    }

    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_board_config_id(target: &Uuid) -> _ {
        crate::schema::ejjoblog::dsl::ejjoblog.filter(ejboard_config_id.eq(target))
    }

    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_job_and_board<'a>(job_id: &'a Uuid, board_config_id: &'a Uuid) -> _ {
        crate::schema::ejjoblog::dsl::ejjoblog
            .filter(ejjob_id.eq(job_id))
            .filter(ejboard_config_id.eq(board_config_id))
    }
}
