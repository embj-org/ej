use std::net::SocketAddr;

use crate::{ej_message::EjServerMessage, web::ctx::ctx_client::CtxClient};
use tokio::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub struct EjConnectedBuilder {
    pub builder: CtxClient,
    pub tx: Sender<EjServerMessage>,
    pub addr: SocketAddr,
}
