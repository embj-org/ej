use crate::ej_config::db::ej_config_db::EjConfigDb;
use crate::prelude::*;
use crate::schema::ejboard::dsl::*;
use crate::{db::connection::DbConnection, schema::ejboard};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

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
    pub id: Uuid,
    pub ejconfig_id: Uuid,
    pub name: String,
    pub description: String,
}

impl NewEjBoardDb {
    pub fn new(
        board_id: Uuid,
        config_id: Uuid,
        board_name: String,
        board_description: String,
    ) -> Self {
        Self {
            id: board_id,
            ejconfig_id: config_id,
            name: board_name,
            description: board_description,
        }
    }

    pub fn save(self, connection: &DbConnection) -> Result<EjBoardDb> {
        use crate::schema::ejboard::dsl::*;
        let conn = &mut connection.pool.get()?;
        Ok(diesel::insert_into(ejboard)
            .values(&self)
            .returning(EjBoardDb::as_returning())
            .get_result(conn)?
            .into())
    }
}

impl EjBoardDb {
    pub fn fetch_by_id(target: &Uuid, connection: &DbConnection) -> Result<Self> {
        use crate::schema::ejboard::dsl::*;
        let conn = &mut connection.pool.get()?;
        Ok(ejboard
            .filter(id.eq(target))
            .select(EjBoardDb::as_select())
            .get_result(conn)?)
    }

    pub fn fetch_by_ejconfig_id(target: &Uuid, connection: &DbConnection) -> Result<Vec<Self>> {
        use crate::schema::ejboard::dsl::*;
        let conn = &mut connection.pool.get()?;
        Ok(ejboard
            .filter(ejconfig_id.eq(target))
            .select(EjBoardDb::as_select())
            .load(conn)?)
    }

    pub fn fetch_by_name(target_name: &str, connection: &DbConnection) -> Result<Vec<Self>> {
        use crate::schema::ejboard::dsl::*;
        let conn = &mut connection.pool.get()?;
        Ok(ejboard
            .filter(name.eq(target_name))
            .select(EjBoardDb::as_select())
            .load(conn)?)
    }
}

impl EjBoardDb {
    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_id(target: &Uuid) -> _ {
        crate::schema::ejboard::dsl::ejboard.filter(id.eq(target))
    }

    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_ejconfig_id(target: &Uuid) -> _ {
        use crate::schema::ejboard::dsl::*;
        crate::schema::ejboard::dsl::ejboard.filter(ejconfig_id.eq(target))
    }

    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_name(target: &str) -> _ {
        use crate::schema::ejboard::dsl::*;
        crate::schema::ejboard::dsl::ejboard.filter(name.eq(target))
    }
}
