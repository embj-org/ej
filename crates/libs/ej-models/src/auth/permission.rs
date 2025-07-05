//! Permission model for authorization.

use crate::{db::connection::DbConnection, prelude::*};
use diesel::prelude::*;

/// A system permission that can be granted to clients.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Hash, PartialEq, Eq)]
#[diesel(table_name = crate::schema::permission)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Permission {
    /// The unique permission identifier.
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
    /// Creates a new permission with the given ID.
    pub const fn new(id: String) -> Self {
        Permission { id }
    }

    /// Fetches all permissions from the database.
    pub fn fetch_all(conn: &DbConnection) -> Result<Vec<Self>> {
        let conn = &mut conn.pool.get()?;
        Ok(crate::schema::permission::table.load(conn)?)
    }

    /// Fetches a permission by its ID.
    pub fn fetch_by_id(conn: &DbConnection, target_id: &String) -> Result<Self> {
        use crate::schema::permission::dsl::*;
        let conn = &mut conn.pool.get()?;
        Ok(permission
            .filter(id.eq(target_id))
            .select(Permission::as_select())
            .get_result(conn)?)
    }
}
