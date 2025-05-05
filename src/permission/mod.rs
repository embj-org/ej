use crate::db::connection::DbConnection;
use crate::prelude::*;
use chrono::{DateTime, Utc};
use diesel::prelude::*;

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Hash, PartialEq, Eq)]
#[diesel(table_name = crate::schema::permission)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Permission {
    pub id: String,
}
impl From<&str> for Permission {
    fn from(value: &str) -> Self {
        Self {
            id: String::from(value),
        }
    }
}

impl Permission {
    fn fetch_all(conn: &DbConnection) -> Result<Vec<Self>> {
        let conn = &mut conn.pool.get()?;
        Ok(crate::schema::permission::table.load(conn)?)
    }
    fn fetch_by_id(conn: &DbConnection, target_id: &String) -> Result<Self> {
        use crate::schema::permission::dsl::*;
        let conn = &mut conn.pool.get()?;
        Ok(permission
            .filter(id.eq(target_id))
            .select(Permission::as_select())
            .get_result(conn)?)
    }
}
