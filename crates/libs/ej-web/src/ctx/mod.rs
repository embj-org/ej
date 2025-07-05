//! Request context management for web handlers.
//!
//! This module provides context structures and utilities for managing
//! authenticated requests and client information in web handlers.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ctx::ctx_client::CtxClient;

pub mod ctx_client;
pub mod resolver;

/// Represents the type of authenticated entity.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum CtxWho {
    /// A regular client.
    Client = 0,
    /// A builder instance.
    Builder = 1,
}

/// Request context containing authentication and authorization information.
#[derive(Clone, Debug)]
pub struct Ctx {
    /// The authenticated client.
    pub client: CtxClient,
    /// Permissions granted to this context.
    pub permissions: HashSet<String>,
    /// Type of authenticated entity (client or builder).
    pub who: CtxWho,
}

impl Ctx {
    /// Creates a new request context.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ej_web::ctx::{Ctx, CtxWho};
    /// use std::collections::HashSet;
    /// use uuid::Uuid;
    ///
    /// let client_id = Uuid::new_v4();
    /// let mut permissions = HashSet::new();
    /// permissions.insert("read".to_string());
    /// permissions.insert("write".to_string());
    ///
    /// let ctx = Ctx::new(client_id, CtxWho::Client, permissions);
    /// assert_eq!(ctx.client.id, client_id);
    /// assert_eq!(ctx.who, CtxWho::Client);
    /// ```
    pub fn new(id: Uuid, who: CtxWho, permissions: HashSet<String>) -> Self {
        Self {
            client: CtxClient { id },
            who,
            permissions,
        }
    }
}
