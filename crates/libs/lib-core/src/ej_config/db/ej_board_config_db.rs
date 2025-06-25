use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::prelude::*;
use crate::{db::connection::DbConnection, ej_config::db::ej_board_db::EjBoardDb};

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, Clone)]
#[diesel(belongs_to(EjBoardDb, foreign_key = ejboard_id))]
#[diesel(table_name = crate::schema::ejboard_config)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EjBoardConfigDb {
    pub id: Uuid,
    pub ejboard_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::ejboard_config)]
pub struct NewEjBoardConfigDb {
    pub id: Uuid,
    pub ejboard_id: Uuid,
    pub name: String,
}

impl NewEjBoardConfigDb {
    pub fn new(id: Uuid, ejboard_id: Uuid, name: String) -> Self {
        Self {
            id,
            ejboard_id,
            name,
        }
    }
    pub fn save(self, connection: &mut DbConnection) -> Result<EjBoardConfigDb> {
        use crate::schema::ejboard_config::dsl::*;
        let conn = &mut connection.pool.get()?;

        Ok(diesel::insert_into(ejboard_config)
            .values(&self)
            .returning(EjBoardConfigDb::as_returning())
            .get_result(conn)?
            .into())
    }
}
