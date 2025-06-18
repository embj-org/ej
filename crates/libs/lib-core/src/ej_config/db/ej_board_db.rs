use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::connection::DbConnection;
use crate::ej_config::db::ej_config_db::EjConfigDb;
use crate::prelude::*;

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, Clone)]
#[diesel(belongs_to(EjConfigDb, foreign_key = ejconfig_id))]
#[diesel(table_name = crate::schema::ejboard)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EjBoardDb {
    pub id: Uuid,
    pub ejconfig_id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::ejboard)]
pub struct NewEjBoardDb {
    pub ejconfig_id: Uuid,
    pub name: String,
    pub description: String,
}
impl NewEjBoardDb {
    pub fn new(ejconfig_id: Uuid, name: String, description: String) -> Self {
        Self {
            ejconfig_id,
            name,
            description,
        }
    }
    pub fn save(self, connection: &mut DbConnection) -> Result<EjBoardDb> {
        use crate::schema::ejboard::dsl::*;
        let conn = &mut connection.pool.get()?;

        Ok(diesel::insert_into(ejboard)
            .values(&self)
            .returning(EjBoardDb::as_returning())
            .get_result(conn)?
            .into())
    }
}
