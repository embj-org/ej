use serde::{Deserialize, Serialize};

use super::ej_board_config::EjBoardConfig;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjBoard {
    pub name: String,
    pub description: String,
    pub configs: Vec<EjBoardConfig>,
}
