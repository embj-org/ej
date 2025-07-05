//! Client authentication and management types.

use std::{collections::HashSet, fmt};

use crate::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Client API representation.
#[derive(Debug, Serialize, Deserialize)]
pub struct EjClientApi {
    /// Unique client identifier.
    pub id: Uuid,
    /// Client name.
    pub name: String,
}

/// Client registration data.
#[derive(Debug, Deserialize, Serialize)]
pub struct EjClientPost {
    /// Client name for registration.
    pub name: String,
    /// Client secret for authentication.
    pub secret: String,
}

/// Client login request.
#[derive(Debug, Deserialize, Serialize)]
pub struct EjClientLoginRequest {
    /// Client name.
    pub name: String,
    /// Client secret.
    pub secret: String,
}

/// Client login response.
#[derive(Debug, Deserialize, Serialize)]
pub struct EjClientLogin {
    /// JWT access token.
    pub access_token: String,
    /// Token type (usually "Bearer").
    pub token_type: String,
}

impl EjClientLoginRequest {
    /// Create a new client login request.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ej_dispatcher_sdk::ejclient::EjClientLoginRequest;
    ///
    /// let login_request = EjClientLoginRequest::new("my_client", "secret_key");
    /// assert_eq!(login_request.name, "my_client");
    /// assert_eq!(login_request.secret, "secret_key");
    /// ```
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
