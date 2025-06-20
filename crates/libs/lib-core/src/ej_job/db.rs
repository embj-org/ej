use super::status::db::EjJobStatus;
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
pub struct EjJob {
    pub id: Uuid,
    pub commit_hash: String,
    pub remote_url: String,
    pub build_status: Option<i32>,
    pub run_status: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, PartialEq, Debug, Clone, Deserialize)]
#[diesel(table_name = crate::schema::ejjob)]
pub struct EjJobCreate {
    pub commit_hash: String,
    pub remote_url: String,
    pub build_status: Option<i32>,
    pub run_status: Option<i32>,
}

impl EjJobCreate {
    pub fn save(self, connection: &DbConnection) -> Result<EjJob> {
        let conn = &mut connection.pool.get()?;
        Ok(diesel::insert_into(ejjob)
            .values(&self)
            .returning(EjJob::as_returning())
            .get_result(conn)?
            .into())
    }
}

impl EjJob {
    pub fn fetch_by_id(target: &Uuid, connection: &DbConnection) -> Result<Self> {
        let conn = &mut connection.pool.get()?;
        let job: EjJob = EjJob::by_id(target)
            .select(EjJob::as_select())
            .get_result(conn)?;
        Ok(job.into())
    }

    pub fn fetch_by_commit_hash(target: &str, connection: &DbConnection) -> Result<Self> {
        let conn = &mut connection.pool.get()?;
        let job: EjJob = EjJob::by_commit_hash(target)
            .select(EjJob::as_select())
            .get_result(conn)?;
        Ok(job.into())
    }

    pub fn fetch_build_status(&self, connection: &DbConnection) -> Result<Option<EjJobStatus>> {
        match self.build_status {
            Some(status_id) => Ok(Some(EjJobStatus::fetch_by_id(&status_id, connection)?)),
            None => Ok(None),
        }
    }

    pub fn fetch_run_status(&self, connection: &DbConnection) -> Result<Option<EjJobStatus>> {
        match self.run_status {
            Some(status_id) => Ok(Some(EjJobStatus::fetch_by_id(&status_id, connection)?)),
            None => Ok(None),
        }
    }

    pub fn fetch_all(connection: &DbConnection) -> Result<Vec<Self>> {
        let conn = &mut connection.pool.get()?;
        Ok(EjJob::table().select(EjJob::as_select()).load(conn)?)
    }

    pub fn update_build_status(&self, new_status: i32, connection: &DbConnection) -> Result<Self> {
        let conn = &mut connection.pool.get()?;
        Ok(diesel::update(EjJob::by_id(&self.id))
            .set(build_status.eq(new_status))
            .returning(EjJob::as_returning())
            .get_result(conn)?
            .into())
    }

    pub fn update_run_status(&self, new_status: i32, connection: &DbConnection) -> Result<Self> {
        let conn = &mut connection.pool.get()?;
        Ok(diesel::update(EjJob::by_id(&self.id))
            .set(run_status.eq(new_status))
            .returning(EjJob::as_returning())
            .get_result(conn)?
            .into())
    }
}

impl EjJob {
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
