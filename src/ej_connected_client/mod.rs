use std::net::SocketAddr;

use crate::{ej_client::EjClient, ej_message::EjServerMessage};
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

#[derive(Clone)]
pub struct EjConnectedClient {
    pub id: Uuid,
    pub tx: Sender<EjServerMessage>,
    pub addr: SocketAddr,
}

impl EjClient {
    pub fn connect(self, tx: Sender<EjServerMessage>, addr: SocketAddr) -> EjConnectedClient {
        EjConnectedClient {
            id: self.id,
            tx,
            addr,
        }
    }
}
