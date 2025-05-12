use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjBoardConfig {
    pub description: String,
    pub tags: Vec<String>,
    pub build_script: String,
    pub run_script: String,
    pub results_path: String,
    pub library_path: String,
}
