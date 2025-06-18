use std::net::SocketAddr;

use crate::{ctx::ctx_client::CtxClient, ej_message::EjServerMessage};
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
pub struct EjConnectedBuilder {
    pub client: CtxClient,
    pub tx: Sender<EjServerMessage>,
    pub addr: SocketAddr,
}

impl CtxClient {
    pub fn connect(self, tx: Sender<EjServerMessage>, addr: SocketAddr) -> EjConnectedBuilder {
        EjConnectedBuilder {
            client: self,
            tx,
            addr,
        }
    }
}
