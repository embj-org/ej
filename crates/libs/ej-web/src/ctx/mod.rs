use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ctx::ctx_client::CtxClient;

pub mod ctx_client;
pub mod resolver;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum CtxWho {
    Client = 0,
    Builder = 1,
}
#[derive(Clone, Debug)]
pub struct Ctx {
    pub client: CtxClient,
    pub permissions: HashSet<String>,
    pub who: CtxWho,
}

impl Ctx {
    pub fn new(id: Uuid, who: CtxWho, permissions: HashSet<String>) -> Self {
        Self {
            client: CtxClient { id },
            who,
            permissions,
        }
    }
}
