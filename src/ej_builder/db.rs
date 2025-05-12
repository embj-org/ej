use crate::prelude::*;
use crate::{db::connection::DbConnection, schema::ejbuilder::dsl::*};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, PartialEq, Serialize, Deserialize)]
#[diesel(belongs_to(EjClient))]
#[diesel(table_name = crate::schema::ejbuilder)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EjBuilder {
    pub id: Uuid,
    pub ejclient_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, PartialEq, Debug, Clone, Deserialize)]
#[diesel(table_name = crate::schema::ejbuilder)]
pub struct EjBuilderCreate {
    pub ejclient_id: Uuid,
}

impl EjBuilderCreate {
    pub fn new(client_id: Uuid) -> Self {
        Self {
            ejclient_id: client_id,
        }
    }
    pub fn create(self, connection: &DbConnection) -> Result<EjBuilder> {
        let conn = &mut connection.pool.get()?;

        Ok(diesel::insert_into(ejbuilder)
            .values(self)
            .returning(EjBuilder::as_returning())
            .get_result(conn)?)
    }
}
impl EjBuilder {
    pub fn fetch_by_id(target: &Uuid, connection: &DbConnection) -> Result<Self> {
        let conn = &mut connection.pool.get()?;

        let client: EjBuilder = EjBuilder::by_id(target)
            .select(EjBuilder::as_select())
            .get_result(conn)?;

        Ok(client.into())
    }
}

impl EjBuilder {
    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn by_id(target: &Uuid) -> _ {
        crate::schema::ejbuilder::dsl::ejbuilder.filter(id.eq(target))
    }
}
