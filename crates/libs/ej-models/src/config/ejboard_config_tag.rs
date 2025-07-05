//! Board config tag associations for organizing configurations.

use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    config::{ejboard_config::EjBoardConfigDb, ejtag::EjTag},
    db::connection::DbConnection,
    prelude::*,
};

/// Associates a board config with a tag for organization and filtering.
#[derive(Queryable, Selectable, Associations, Debug, Clone)]
#[diesel(belongs_to(EjBoardConfigDb, foreign_key = ejboard_config_id))]
#[diesel(belongs_to(EjTag, foreign_key = ejtag_id))]
#[diesel(table_name = crate::schema::ejboard_config_tag)]
#[diesel(primary_key(ejboard_config_id, ejtag_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EjBoardConfigTag {
    /// The board config ID.
    pub ejboard_config_id: Uuid,
    /// The tag ID.
    pub ejtag_id: Uuid,
}

/// Data for creating a new board config tag association.
#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::ejboard_config_tag)]
pub struct NewEjBoardConfigTag {
    /// The board config ID to tag.
    pub ejboard_config_id: Uuid,
    /// The tag ID to associate.
    pub ejtag_id: Uuid,
}

impl NewEjBoardConfigTag {
    /// Creates a new board config tag association.
    pub fn new(ejboard_config_id: Uuid, ejtag_id: Uuid) -> Self {
        Self {
            ejboard_config_id,
            ejtag_id,
        }
    }

    /// Saves the association to the database.
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

impl EjBoardConfigTag {
    /// Retrieve a board config with all its associated tags.
    pub fn fetch_by_board_config(
        board_config_id: Uuid,
        connection: &DbConnection,
    ) -> Result<(EjBoardConfigDb, Vec<EjTag>)> {
        use crate::schema::{ejboard_config, ejboard_config_tag, ejtag};

        let conn = &mut connection.pool.get()?;

        let board_config = ejboard_config::table
            .find(board_config_id)
            .first::<EjBoardConfigDb>(conn)?;

        let tags = ejboard_config_tag::table
            .inner_join(ejtag::table.on(ejboard_config_tag::ejtag_id.eq(ejtag::id)))
            .filter(ejboard_config_tag::ejboard_config_id.eq(board_config_id))
            .select(EjTag::as_select())
            .load::<EjTag>(conn)?;

        Ok((board_config, tags))
    }

    /// Retrieve a tag with all board configs that have this tag.
    pub fn fetch_by_tag(
        tag_id: Uuid,
        connection: &DbConnection,
    ) -> Result<(EjTag, Vec<EjBoardConfigDb>)> {
        use crate::schema::{ejboard_config, ejboard_config_tag, ejtag};
        let conn = &mut connection.pool.get()?;

        let tag = ejtag::table.find(tag_id).first::<EjTag>(conn)?;

        let board_configs = ejboard_config_tag::table
            .inner_join(
                ejboard_config::table
                    .on(ejboard_config_tag::ejboard_config_id.eq(ejboard_config::id)),
            )
            .filter(ejboard_config_tag::ejtag_id.eq(tag_id))
            .select(EjBoardConfigDb::as_select())
            .load::<EjBoardConfigDb>(conn)?;

        Ok((tag, board_configs))
    }
}
