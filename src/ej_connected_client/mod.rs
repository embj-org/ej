use std::net::SocketAddr;

use crate::{ctx::ctx_client::CtxClient, ej_message::EjServerMessage};
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
