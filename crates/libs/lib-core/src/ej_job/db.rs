use super::status::db::EjJobStatus;
use crate::ej_job::job_type::db::EjJobTypeDb;
use crate::prelude::*;
use crate::{db::connection::DbConnection, schema::ejjob::dsl::*};
use chrono::{DateTime, Utc};
use diesel::associations::HasTable;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, PartialEq, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::ejjob)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EjJobDb {
    pub id: Uuid,
    pub commit_hash: String,
    pub remote_url: String,
    pub job_type: i32,
    pub status: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, PartialEq, Debug, Clone, Deserialize)]
#[diesel(table_name = crate::schema::ejjob)]
pub struct EjJobCreate {
    pub commit_hash: String,
    pub remote_url: String,
}

impl EjJobCreate {
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

    pub fn fetch_by_commit_hash(target: &str, connection: &DbConnection) -> Result<Self> {
        let conn = &mut connection.pool.get()?;
        let job: EjJobDb = EjJobDb::by_commit_hash(target)
            .select(EjJobDb::as_select())
            .get_result(conn)?;
        Ok(job.into())
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
