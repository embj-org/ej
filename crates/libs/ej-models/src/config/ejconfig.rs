//! Configuration model for managing config versions and hashes.

use crate::db::connection::DbConnection;
use crate::prelude::*;
use crate::schema::ejconfig::dsl::*;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

/// A configuration version associated with a builder.
#[derive(Queryable, Selectable, Identifiable, Debug, Clone)]
#[diesel(table_name = crate::schema::ejconfig)]
#[diesel(belongs_to(EjBuilder))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EjConfigDb {
    /// Unique config ID.
    pub id: Uuid,
    /// The builder this config belongs to.
    pub ejbuilder_id: Uuid,
    /// Configuration hash for integrity verification.
    pub hash: String,
    /// Configuration version.
    pub version: String,
    /// When this config was created.
    pub created_at: DateTime<Utc>,
    /// When this config was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Data for creating a new config entry.
#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::ejconfig)]
pub struct NewEjConfigDb {
    /// The builder ID this config belongs to.
    pub ejbuilder_id: Uuid,
    /// Configuration version.
    pub version: String,
    /// Configuration hash.
    pub hash: String,
}

impl EjConfigDb {
    /// Fetches a client's config by ID and hash.
    pub fn fetch_client_config(
        connection: &mut DbConnection,
        client_id: &Uuid,
        config_hash: &str,
    ) -> Result<Self> {
        let conn = &mut connection.pool.get()?;
        Ok(EjConfigDb::client_config(client_id, config_hash)
            .select(EjConfigDb::as_select())
            .first(conn)?)
    }
}

impl NewEjConfigDb {
    pub fn new(builder_id: Uuid, config_version: String, config_hash: String) -> Self {
        Self {
            ejbuilder_id: builder_id,
            version: config_version,
            hash: config_hash,
        }
    }
    pub fn save(self, connection: &mut DbConnection) -> Result<EjConfigDb> {
        use crate::schema::ejconfig::dsl::*;
        let conn = &mut connection.pool.get()?;

        Ok(diesel::insert_into(ejconfig)
            .values(&self)
            .returning(EjConfigDb::as_returning())
            .get_result(conn)?
            .into())
    }
}

impl EjConfigDb {
    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_id(target: &Uuid) -> _ {
        crate::schema::ejconfig::dsl::ejconfig.filter(id.eq(target))
    }

    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn client_config<'a>(client_id: &'a Uuid, config_hash: &'a str) -> _ {
        crate::schema::ejconfig::dsl::ejconfig
            .filter(ejbuilder_id.eq(client_id))
            .filter(hash.eq(config_hash))
    }
}
