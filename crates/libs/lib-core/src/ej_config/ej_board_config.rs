use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjBoardConfig {
    pub name: String,
    pub tags: Vec<String>,
    pub build_script: String,
    pub run_script: String,
    pub results_path: String,
    pub library_path: String,
}
