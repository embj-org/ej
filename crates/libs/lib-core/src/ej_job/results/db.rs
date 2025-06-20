use crate::ej_job::db::EjJobDb;
use crate::prelude::*;
use crate::{db::connection::DbConnection, schema::ejjobresult::dsl::*};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Selectable, PartialEq, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::ejjobresult)]
#[diesel(belongs_to(EjJob))]
#[diesel(belongs_to(EjBoardConfig))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EjJobResult {
    pub ejjob_id: Uuid,
    pub ejboard_config_id: Uuid,
    pub result: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, PartialEq, Debug, Clone, Deserialize)]
#[diesel(table_name = crate::schema::ejjobresult)]
pub struct EjJobResultCreate {
    pub ejjob_id: Uuid,
    pub ejboard_config_id: Uuid,
    pub result: String,
}

impl EjJobResultCreate {
    pub fn save(self, connection: &DbConnection) -> Result<EjJobResult> {
        let conn = &mut connection.pool.get()?;
        Ok(diesel::insert_into(ejjobresult)
            .values(&self)
            .returning(EjJobResult::as_returning())
            .get_result(conn)?
            .into())
    }
}

impl EjJobResult {
    pub fn fetch_by_composite_key(
        job_id: &Uuid,
        board_config_id: &Uuid,
        connection: &DbConnection,
    ) -> Result<Self> {
        let conn = &mut connection.pool.get()?;
        let job_result: EjJobResult = EjJobResult::by_composite_key(job_id, board_config_id)
            .select(EjJobResult::as_select())
            .get_result(conn)?;
        Ok(job_result)
    }

    pub fn fetch_by_job_id(target: &Uuid, connection: &DbConnection) -> Result<Vec<Self>> {
        let conn = &mut connection.pool.get()?;
        Ok(EjJobResult::by_job_id(target)
            .select(EjJobResult::as_select())
            .load(conn)?)
    }

    pub fn fetch_by_board_config_id(target: &Uuid, connection: &DbConnection) -> Result<Vec<Self>> {
        let conn = &mut connection.pool.get()?;
        Ok(EjJobResult::by_board_config_id(target)
            .select(EjJobResult::as_select())
            .load(conn)?)
    }

    pub fn fetch_job(&self, connection: &DbConnection) -> Result<EjJobDb> {
        EjJobDb::fetch_by_id(&self.ejjob_id, connection)
    }

    pub fn update_result(&self, new_result: String, connection: &DbConnection) -> Result<Self> {
        let conn = &mut connection.pool.get()?;
        Ok(diesel::update(EjJobResult::by_composite_key(
            &self.ejjob_id,
            &self.ejboard_config_id,
        ))
        .set(result.eq(new_result))
        .returning(EjJobResult::as_returning())
        .get_result(conn)?
        .into())
    }

    pub fn delete(&self, connection: &DbConnection) -> Result<()> {
        let conn = &mut connection.pool.get()?;
        diesel::delete(EjJobResult::by_composite_key(
            &self.ejjob_id,
            &self.ejboard_config_id,
        ))
        .execute(conn)?;
        Ok(())
    }
}

impl EjJobResult {
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
