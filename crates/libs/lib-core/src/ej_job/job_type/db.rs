use crate::prelude::*;
use crate::{db::connection::DbConnection, schema::ejjobtype::dsl::*};
use diesel::associations::HasTable;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, PartialEq, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::ejjobtype)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EjJobTypeDb {
    pub id: i32,
    pub job_type: String,
}

#[derive(Insertable, PartialEq, Debug, Clone, Deserialize)]
#[diesel(table_name = crate::schema::ejjobtype)]
pub struct EjJobTypeCreate {
    pub id: i32,
    pub job_type: String,
}

impl EjJobTypeDb {
    pub fn build() -> i32 {
        0
    }
    pub fn run() -> i32 {
        1
    }
}
impl EjJobTypeCreate {
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
    pub fn fetch_by_id(target: i32, connection: &DbConnection) -> Result<Self> {
        let conn = &mut connection.pool.get()?;
        let job_status: EjJobTypeDb = EjJobTypeDb::by_id(target)
            .select(EjJobTypeDb::as_select())
            .get_result(conn)?;
        Ok(job_status.into())
    }

    pub fn fetch_by_status(target: &str, connection: &DbConnection) -> Result<Self> {
        let conn = &mut connection.pool.get()?;
        let job_status: EjJobTypeDb = EjJobTypeDb::by_status(target)
            .select(EjJobTypeDb::as_select())
            .get_result(conn)?;
        Ok(job_status.into())
    }

    pub fn fetch_all(connection: &DbConnection) -> Result<Vec<Self>> {
        let conn = &mut connection.pool.get()?;
        Ok(EjJobTypeDb::table()
            .select(EjJobTypeDb::as_select())
            .load(conn)?)
    }
}

impl EjJobTypeDb {
    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_id(target: i32) -> _ {
        crate::schema::ejjobtype::dsl::ejjobtype.filter(id.eq(target))
    }

    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_status(target: &str) -> _ {
        crate::schema::ejjobtype::dsl::ejjobtype.filter(job_type.eq(target))
    }
}
