use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjUserBoardConfig {
    pub name: String,
    pub tags: Vec<String>,
    pub build_script: String,
    pub run_script: String,
    pub results_path: String,
    pub library_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjBoardConfig {
    pub id: Uuid,
    pub name: String,
    pub tags: Vec<String>,
    pub build_script: String,
    pub run_script: String,
    pub results_path: String,
    pub library_path: String,
}
impl EjBoardConfig {
    pub fn from_ej_board_config(value: EjUserBoardConfig) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: value.name,
            tags: value.tags,
            build_script: value.build_script,
            run_script: value.run_script,
            results_path: value.results_path,
            library_path: value.library_path,
        }
    }
}
