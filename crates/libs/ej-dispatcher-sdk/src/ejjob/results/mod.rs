//! Job result types and utilities.

use std::collections::HashMap;

use ej_config::ej_config::EjConfig;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Board configuration identifier type alias.
pub type EjBoardConfigId = Uuid;

/// Build result from a specific builder.
#[derive(Debug, Serialize, Deserialize)]
pub struct EjBuilderBuildResult {
    /// Job identifier.
    pub job_id: Uuid,
    /// Builder identifier.
    pub builder_id: Uuid,
    /// Build logs per board configuration.
    pub logs: HashMap<EjBoardConfigId, Vec<String>>,
    /// Whether the build was successful.
    pub successful: bool,
}

/// Run result from a specific builder.
#[derive(Debug, Serialize, Deserialize)]
pub struct EjBuilderRunResult {
    /// Job identifier.
    pub job_id: Uuid,
    /// Builder identifier.
    pub builder_id: Uuid,
    /// Run logs per board configuration.
    pub logs: HashMap<EjBoardConfigId, Vec<String>>,
    /// Run results per board configuration.
    pub results: HashMap<EjBoardConfigId, String>,
    /// Whether the run was successful.
    pub successful: bool,
}
