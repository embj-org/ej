use crate::prelude::*;
use crate::{db::connection::DbConnection, schema::ejjobstatus::dsl::*};
use diesel::associations::HasTable;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, PartialEq, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::ejjobstatus)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EjJobStatus {
    pub id: i32,
    pub status: String,
}

#[derive(Insertable, PartialEq, Debug, Clone, Deserialize)]
#[diesel(table_name = crate::schema::ejjobstatus)]
pub struct EjJobStatusCreate {
    pub id: i32,
    pub status: String,
}

impl EjJobStatusCreate {
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
    pub fn fetch_by_id(target: &i32, connection: &DbConnection) -> Result<Self> {
        let conn = &mut connection.pool.get()?;
        let job_status: EjJobStatus = EjJobStatus::by_id(target)
            .select(EjJobStatus::as_select())
            .get_result(conn)?;
        Ok(job_status.into())
    }

    pub fn fetch_by_status(target: &str, connection: &DbConnection) -> Result<Self> {
        let conn = &mut connection.pool.get()?;
        let job_status: EjJobStatus = EjJobStatus::by_status(target)
            .select(EjJobStatus::as_select())
            .get_result(conn)?;
        Ok(job_status.into())
    }

    pub fn fetch_all(connection: &DbConnection) -> Result<Vec<Self>> {
        let conn = &mut connection.pool.get()?;
        Ok(EjJobStatus::table()
            .select(EjJobStatus::as_select())
            .load(conn)?)
    }
}

impl EjJobStatus {
    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_id(target: &i32) -> _ {
        crate::schema::ejjobstatus::dsl::ejjobstatus.filter(id.eq(target))
    }

    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_status(target: &str) -> _ {
        crate::schema::ejjobstatus::dsl::ejjobstatus.filter(status.eq(target))
    }
}
