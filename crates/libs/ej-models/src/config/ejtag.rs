use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::prelude::*;

use crate::db::connection::DbConnection;
use crate::schema::ejtag::dsl::*;

#[derive(Queryable, Selectable, Identifiable, Debug, Clone)]
#[diesel(table_name = crate::schema::ejtag)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EjTag {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::ejtag)]
pub struct NewEjTag {
    pub name: String,
}

impl EjTag {
    pub fn fetch_by_name(connection: &mut DbConnection, tag_name: &str) -> Result<Self> {
        let conn = &mut connection.pool.get()?;
        let tag = EjTag::by_name(tag_name).first(conn)?;
        Ok(tag)
    }
}

impl NewEjTag {
    pub fn new(tag_name: impl Into<String>) -> Self {
        Self {
            name: tag_name.into(),
        }
    }

    pub fn save(self, connection: &mut DbConnection) -> Result<EjTag> {
        let conn = &mut connection.pool.get()?;

        Ok(diesel::insert_into(ejtag)
            .values(&self)
            .returning(EjTag::as_returning())
            .get_result(conn)?
            .into())
    }
}

impl EjTag {
    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_id(target: &Uuid) -> _ {
        crate::schema::ejtag::dsl::ejtag.filter(id.eq(target))
    }

    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_name(target: &str) -> _ {
        crate::schema::ejtag::dsl::ejtag.filter(name.eq(target))
    }
}
