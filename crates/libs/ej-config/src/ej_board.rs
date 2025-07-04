use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ej_board_config::{EjBoardConfig, EjUserBoardConfig};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjUserBoard {
    pub name: String,
    pub description: String,
    pub configs: Vec<EjUserBoardConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjBoard {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub configs: Vec<EjBoardConfig>,
}
impl EjBoard {
    pub fn from_ej_board(board: EjUserBoard) -> Self {
        let configs: Vec<EjBoardConfig> = board
            .configs
            .into_iter()
            .map(|conf: EjUserBoardConfig| EjBoardConfig::from_ej_board_config(conf))
            .collect();

        Self {
            id: Uuid::new_v4(),
            name: board.name,
            description: board.description,
            configs: configs,
        }
    }
}
