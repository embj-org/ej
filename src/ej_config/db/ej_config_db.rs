use crate::db::connection::DbConnection;
use crate::prelude::*;
use crate::schema::ejconfig::dsl::*;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable, Identifiable, Debug, Clone)]
#[diesel(table_name = crate::schema::ejconfig)]
#[diesel(belongs_to(EjClient))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EjConfigDb {
    pub id: Uuid,
    pub ejclient_id: Uuid,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::ejconfig)]
pub struct NewEjConfigDb {
    pub ejclient_id: Uuid,
    pub version: String,
}

impl EjConfigDb {
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
    pub fn new(client_id: Uuid, config_version: String) -> Self {
        Self {
            ejclient_id: client_id,
            version: config_version,
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
            .filter(ejclient_id.eq(client_id))
            .filter(hash.eq(config_hash))
    }
}
