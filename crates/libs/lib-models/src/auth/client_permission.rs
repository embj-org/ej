use crate::{
    auth::permission::Permission, client::ejclient::EjClient, db::connection::DbConnection,
    prelude::*,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Identifiable, Selectable, Queryable, Associations, Debug)]
#[diesel(belongs_to(EjClient, foreign_key = ejclient_id))]
#[diesel(belongs_to(Permission))]
#[diesel(table_name = crate::schema::client_permission)]
#[diesel(primary_key(ejclient_id, permission_id))]
pub struct ClientPermission {
    pub ejclient_id: Uuid,
    pub permission_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, PartialEq, Debug, Clone, Deserialize)]
#[diesel(table_name = crate::schema::client_permission)]
pub struct NewClientPermission {
    pub ejclient_id: Uuid,
    pub permission_id: String,
}

#[derive(Deserialize)]
pub struct ClientPermissionKey {
    pub ej_client_id: Uuid,
    pub permission_id: String,
}

impl ClientPermission {
    pub fn new(conn: &DbConnection, item: NewClientPermission) -> Result<Self> {
        let connection = &mut conn.pool.get()?;
        Ok(diesel::insert_into(crate::schema::client_permission::table)
            .values(item)
            .returning(ClientPermission::as_returning())
            .get_result(connection)?)
    }

    pub fn fetch_by_id(conn: &DbConnection, key: &ClientPermissionKey) -> Result<Self> {
        use crate::schema::client_permission::dsl::*;
        let conn = &mut conn.pool.get()?;
        Ok(client_permission
            .filter(ejclient_id.eq(key.ej_client_id))
            .filter(permission_id.eq(&key.permission_id))
            .select(ClientPermission::as_select())
            .get_result(conn)?)
    }

    pub fn fetch_by_client<'a>(
        conn: &DbConnection,
        client: &'a EjClient,
    ) -> Result<(&'a EjClient, Vec<Permission>)> {
        let conn = &mut conn.pool.get()?;

        let permissions: Vec<Permission> = ClientPermission::belonging_to(client)
            .inner_join(crate::schema::ejclient::table)
            .inner_join(crate::schema::permission::table)
            .select(Permission::as_select())
            .load(conn)?;

        Ok((client, permissions))
    }

    pub fn fetch_by_permission<'a>(
        conn: &DbConnection,
        permission: &'a Permission,
    ) -> Result<(&'a Permission, Vec<EjClient>)> {
        let conn = &mut conn.pool.get()?;
        let users = ClientPermission::belonging_to(&permission)
            .inner_join(crate::schema::ejclient::table)
            .inner_join(crate::schema::permission::table)
            .select(EjClient::as_select())
            .load(conn)?;

        Ok((permission, users))
    }
}
