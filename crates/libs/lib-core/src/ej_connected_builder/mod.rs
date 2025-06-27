use std::net::SocketAddr;

use crate::{ctx::ctx_client::CtxClient, ej_message::EjServerMessage};
use tokio::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub struct EjConnectedBuilder {
    pub builder: CtxClient,
    pub tx: Sender<EjServerMessage>,
    pub addr: SocketAddr,
}

impl CtxClient {
    pub fn connect(self, tx: Sender<EjServerMessage>, addr: SocketAddr) -> EjConnectedBuilder {
        EjConnectedBuilder {
            builder: self,
            tx,
            addr,
        }
    }
}
