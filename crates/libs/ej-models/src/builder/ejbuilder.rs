//! Builder service model for managing build instances.

use crate::prelude::*;
use crate::{db::connection::DbConnection, schema::ejbuilder::dsl::*};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A builder instance that processes jobs in the ej system.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, PartialEq, Serialize, Deserialize)]
#[diesel(belongs_to(EjClient))]
#[diesel(table_name = crate::schema::ejbuilder)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EjBuilder {
    /// Unique builder ID.
    pub id: Uuid,
    /// The client that owns this builder.
    pub ejclient_id: Uuid,
    /// When this builder was created.
    pub created_at: DateTime<Utc>,
    /// When this builder was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Data for creating a new builder.
#[derive(Insertable, PartialEq, Debug, Clone, Deserialize)]
#[diesel(table_name = crate::schema::ejbuilder)]
pub struct EjBuilderCreate {
    /// The client ID that will own this builder.
    pub ejclient_id: Uuid,
}

impl EjBuilderCreate {
    /// Creates a new builder creation request.
    pub fn new(client_id: Uuid) -> Self {
        Self {
            ejclient_id: client_id,
        }
    }

    /// Creates the builder in the database.
    pub fn create(self, connection: &DbConnection) -> Result<EjBuilder> {
        let conn = &mut connection.pool.get()?;

        Ok(diesel::insert_into(ejbuilder)
            .values(self)
            .returning(EjBuilder::as_returning())
            .get_result(conn)?)
    }
}

impl EjBuilder {
    /// Fetches a builder by its ID.
    pub fn fetch_by_id(target: &Uuid, connection: &DbConnection) -> Result<Self> {
        let conn = &mut connection.pool.get()?;

        let client: EjBuilder = EjBuilder::by_id(target)
            .select(EjBuilder::as_select())
            .get_result(conn)?;

        Ok(client.into())
    }

    /// Returns a query filtered by builder ID.
    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_id(target: &Uuid) -> _ {
        crate::schema::ejbuilder::dsl::ejbuilder.filter(id.eq(target))
    }
}
