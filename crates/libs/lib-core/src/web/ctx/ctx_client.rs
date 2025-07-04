use std::net::SocketAddr;

use crate::{ej_connected_builder::EjConnectedBuilder, ej_message::EjServerMessage, prelude::*};
use lib_models::{builder::ejbuilder::EjBuilderCreate, db::connection::DbConnection};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::ej_builder::api::EjBuilderApi;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtxClient {
    pub id: Uuid,
}

impl CtxClient {
    pub fn create_builder(&self, conn: &mut DbConnection) -> Result<EjBuilderApi> {
        Ok(EjBuilderCreate::new(self.id).create(conn)?.try_into()?)
    }

    pub fn connect(self, tx: Sender<EjServerMessage>, addr: SocketAddr) -> EjConnectedBuilder {
        EjConnectedBuilder {
            builder: self,
            tx,
            addr,
        }
    }
}
