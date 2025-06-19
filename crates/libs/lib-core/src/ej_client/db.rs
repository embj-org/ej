use crate::ej_client_permission::ClientPermission;
use crate::permission::Permission;
use crate::prelude::*;
use crate::schema::ejclient;
use crate::{db::connection::DbConnection, schema::ejclient::dsl::*};
use chrono::{DateTime, Utc};
use diesel::associations::HasTable;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, PartialEq, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::ejclient)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EjClient {
    pub id: Uuid,
    pub name: String,
    pub hash: String,
    pub hash_version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, PartialEq, Debug, Clone, Deserialize)]
#[diesel(table_name = crate::schema::ejclient)]
pub struct EjClientCreate {
    pub name: String,
    pub hash: String,
    pub hash_version: i32,
}

impl EjClientCreate {
    pub fn save(self, connection: &DbConnection) -> Result<EjClient> {
        let conn = &mut connection.pool.get()?;

        Ok(diesel::insert_into(ejclient)
            .values(&self)
            .returning(EjClient::as_returning())
            .get_result(conn)?
            .into())
    }
}

impl EjClient {
    pub fn fetch_by_id(target: &Uuid, connection: &DbConnection) -> Result<Self> {
        let conn = &mut connection.pool.get()?;

        let client: EjClient = EjClient::by_id(target)
            .select(EjClient::as_select())
            .get_result(conn)?;

        Ok(client.into())
    }
    pub fn fetch_by_name(target: &str, connection: &DbConnection) -> Result<Self> {
        let conn = &mut connection.pool.get()?;

        let client: EjClient = EjClient::by_name(target)
            .select(EjClient::as_select())
            .get_result(conn)?;

        Ok(client.into())
    }
    pub fn fetch_permissions(&self, connection: &DbConnection) -> Result<Vec<Permission>> {
        Ok(ClientPermission::fetch_by_client(connection, self)?.1)
    }
    pub fn fetch_all(connection: &DbConnection) -> Result<Vec<Self>> {
        let conn = &mut connection.pool.get()?;

        Ok(EjClient::table().select(EjClient::as_select()).load(conn)?)
    }
}

impl EjClient {
    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_id(target: &Uuid) -> _ {
        crate::schema::ejclient::dsl::ejclient.filter(id.eq(target))
    }

    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_name(target: &str) -> _ {
        crate::schema::ejclient::dsl::ejclient.filter(name.eq(target))
    }
}
