//! Builder registration and management types.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Builder API representation.
#[derive(Debug, Serialize, Deserialize)]
pub struct EjBuilderApi {
    /// Unique builder identifier.
    pub id: Uuid,
    /// Builder authentication token.
    pub token: String,
}
