//! Client permission associations for authorization.

use crate::{
    auth::permission::Permission, client::ejclient::EjClient, db::connection::DbConnection,
    prelude::*,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

/// Associates a client with a specific permission.
#[derive(Identifiable, Selectable, Queryable, Associations, Debug)]
#[diesel(belongs_to(EjClient, foreign_key = ejclient_id))]
#[diesel(belongs_to(Permission))]
#[diesel(table_name = crate::schema::client_permission)]
#[diesel(primary_key(ejclient_id, permission_id))]
pub struct ClientPermission {
    /// The client ID.
    pub ejclient_id: Uuid,
    /// The permission ID.
    pub permission_id: String,
    /// When this association was created.
    pub created_at: DateTime<Utc>,
    /// When this association was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Data for creating a new client permission association.
#[derive(Insertable, PartialEq, Debug, Clone, Deserialize)]
#[diesel(table_name = crate::schema::client_permission)]
pub struct NewClientPermission {
    /// The client ID.
    pub ejclient_id: Uuid,
    /// The permission ID.
    pub permission_id: String,
}

/// Composite key for identifying a client permission.
#[derive(Deserialize)]
pub struct ClientPermissionKey {
    /// The client ID.
    pub ej_client_id: Uuid,
    /// The permission ID.
    pub permission_id: String,
}

impl ClientPermission {
    /// Creates a new client permission association.
    pub fn new(conn: &DbConnection, item: NewClientPermission) -> Result<Self> {
        let connection = &mut conn.pool.get()?;
        Ok(diesel::insert_into(crate::schema::client_permission::table)
            .values(item)
            .returning(ClientPermission::as_returning())
            .get_result(connection)?)
    }

    /// Fetches a client permission by its composite key.
    pub fn fetch_by_id(conn: &DbConnection, key: &ClientPermissionKey) -> Result<Self> {
        use crate::schema::client_permission::dsl::*;
        let conn = &mut conn.pool.get()?;
        Ok(client_permission
            .filter(ejclient_id.eq(key.ej_client_id))
            .filter(permission_id.eq(&key.permission_id))
            .select(ClientPermission::as_select())
            .get_result(conn)?)
    }

    /// Fetches all permissions for a given client.
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

    /// Fetches all clients that have a given permission.
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
