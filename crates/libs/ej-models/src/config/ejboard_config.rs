//! Board configuration model for managing individual board configurations.

use crate::prelude::*;
use crate::schema::ejboard_config::dsl::*;
use crate::{config::ejboard::EjBoardDb, db::connection::DbConnection};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

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
    pub fn new(board_config_id: Uuid, board_id: Uuid, board_config_name: String) -> Self {
        Self {
            id: board_config_id,
            ejboard_id: board_id,
            name: board_config_name,
        }
    }

    pub fn save(self, connection: &DbConnection) -> Result<EjBoardConfigDb> {
        use crate::schema::ejboard_config::dsl::*;
        let conn = &mut connection.pool.get()?;
        Ok(diesel::insert_into(ejboard_config)
            .values(&self)
            .returning(EjBoardConfigDb::as_returning())
            .get_result(conn)?
            .into())
    }
}

impl EjBoardConfigDb {
    pub fn fetch_by_id(target: &Uuid, connection: &DbConnection) -> Result<Self> {
        use crate::schema::ejboard_config::dsl::*;
        let conn = &mut connection.pool.get()?;
        Ok(ejboard_config
            .filter(id.eq(target))
            .select(EjBoardConfigDb::as_select())
            .get_result(conn)?)
    }

    pub fn fetch_by_board_id(target: &Uuid, connection: &DbConnection) -> Result<Vec<Self>> {
        use crate::schema::ejboard_config::dsl::*;
        let conn = &mut connection.pool.get()?;
        Ok(ejboard_config
            .filter(ejboard_id.eq(target))
            .select(EjBoardConfigDb::as_select())
            .load(conn)?)
    }

    pub fn fetch_by_name(target_name: &str, connection: &DbConnection) -> Result<Vec<Self>> {
        use crate::schema::ejboard_config::dsl::*;
        let conn = &mut connection.pool.get()?;
        Ok(ejboard_config
            .filter(name.eq(target_name))
            .select(EjBoardConfigDb::as_select())
            .load(conn)?)
    }

    pub fn fetch_board(&self, connection: &DbConnection) -> Result<EjBoardDb> {
        EjBoardDb::fetch_by_id(&self.ejboard_id, connection)
    }

    pub fn update_name(&self, new_name: String, connection: &DbConnection) -> Result<Self> {
        use crate::schema::ejboard_config::dsl::*;
        let conn = &mut connection.pool.get()?;
        Ok(diesel::update(ejboard_config.filter(id.eq(&self.id)))
            .set(name.eq(new_name))
            .returning(EjBoardConfigDb::as_returning())
            .get_result(conn)?
            .into())
    }

    pub fn delete(&self, connection: &DbConnection) -> Result<()> {
        use crate::schema::ejboard_config::dsl::*;
        let conn = &mut connection.pool.get()?;
        diesel::delete(ejboard_config.filter(id.eq(&self.id))).execute(conn)?;
        Ok(())
    }
}

impl EjBoardConfigDb {
    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_id(target: &Uuid) -> _ {
        crate::schema::ejboard_config::dsl::ejboard_config.filter(id.eq(target))
    }

    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_ejboard_id(target: &Uuid) -> _ {
        crate::schema::ejboard_config::dsl::ejboard_config.filter(ejboard_id.eq(target))
    }

    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_name(target: &str) -> _ {
        crate::schema::ejboard_config::dsl::ejboard_config.filter(name.eq(target))
    }
}
