use std::sync::Arc;

use ej::{db::connection::DbConnection, ej_connected_builder::EjConnectedBuilder};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Dispatcher {
    pub builders: Arc<Mutex<Vec<EjConnectedBuilder>>>,
    pub connection: DbConnection,
}
impl Dispatcher {
    pub fn new(connection: DbConnection) -> Self {
        Self {
            connection,
            builders: Arc::new(Mutex::new(Vec::new())),
        }
    }
}
