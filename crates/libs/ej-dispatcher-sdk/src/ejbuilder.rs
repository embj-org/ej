use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct EjBuilderApi {
    pub id: Uuid,
    pub token: String,
}
