use std::net::SocketAddr;

use ej_dispatcher_sdk::ejws_message::EjWsServerMessage;
use tokio::sync::mpsc::Sender;

use crate::ctx::ctx_client::CtxClient;

#[derive(Debug, Clone)]
pub struct EjConnectedBuilder {
    pub builder: CtxClient,
    pub tx: Sender<EjWsServerMessage>,
    pub addr: SocketAddr,
}
