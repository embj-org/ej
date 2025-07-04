use std::{collections::HashSet, fmt};

use crate::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct EjClientApi {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EjClientPost {
    pub name: String,
    pub secret: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EjClientLoginRequest {
    pub name: String,
    pub secret: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EjClientLogin {
    pub access_token: String,
    pub token_type: String,
}

impl EjClientLoginRequest {
    pub fn new(name: impl Into<String>, secret: impl Into<String>) -> Self {
        let name = name.into();
        let secret = secret.into();
        Self { name, secret }
    }
}

impl fmt::Display for EjClientApi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Client '{}' (ID: {})", self.name, self.id)
    }
}
