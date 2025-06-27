use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    db::connection::DbConnection,
    ej_config::db::{ej_board_config_db::EjBoardConfigDb, ej_tag::EjTag},
    prelude::*,
};

#[derive(Queryable, Selectable, Associations, Debug, Clone)]
#[diesel(belongs_to(EjBoardConfigDb, foreign_key = ejboard_config_id))]
#[diesel(belongs_to(EjTag, foreign_key = ejtag_id))]
#[diesel(table_name = crate::schema::ejboard_config_tag)]
#[diesel(primary_key(ejboard_config_id, ejtag_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EjBoardConfigTag {
    pub ejboard_config_id: Uuid,
    pub ejtag_id: Uuid,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::ejboard_config_tag)]
pub struct NewEjBoardConfigTag {
    pub ejboard_config_id: Uuid,
    pub ejtag_id: Uuid,
}

impl NewEjBoardConfigTag {
    pub fn new(ejboard_config_id: Uuid, ejtag_id: Uuid) -> Self {
        Self {
            ejboard_config_id,
            ejtag_id,
        }
    }
    pub fn save(self, connection: &mut DbConnection) -> Result<EjBoardConfigTag> {
        use crate::schema::ejboard_config_tag::dsl::*;
        let conn = &mut connection.pool.get()?;

        Ok(diesel::insert_into(ejboard_config_tag)
            .values(&self)
            .returning(EjBoardConfigTag::as_returning())
            .get_result(conn)?
            .into())
    }
}
