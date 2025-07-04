use std::collections::HashMap;

use ej_config::ej_config::EjConfig;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type EjBoardConfigId = Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct EjBuilderBuildResult {
    pub job_id: Uuid,
    pub builder_id: Uuid,
    pub logs: HashMap<EjBoardConfigId, Vec<String>>,
    pub successful: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EjBuilderRunResult {
    pub job_id: Uuid,
    pub builder_id: Uuid,
    pub logs: HashMap<EjBoardConfigId, Vec<String>>,
    pub results: HashMap<EjBoardConfigId, String>,
    pub successful: bool,
}
