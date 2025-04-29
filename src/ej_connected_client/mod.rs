use std::net::SocketAddr;

use crate::{ej_message::EjServerMessage, web::ctx::CtxClient};
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
pub struct EjConnectedClient {
    pub client: CtxClient,
    pub tx: Sender<EjServerMessage>,
    pub addr: SocketAddr,
}

impl CtxClient {
    pub fn connect(self, tx: Sender<EjServerMessage>, addr: SocketAddr) -> EjConnectedClient {
        EjConnectedClient {
            client: self,
            tx,
            addr,
        }
    }
}
