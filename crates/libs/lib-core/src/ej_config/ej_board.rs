use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ej_config::ej_board_config::EjDispatcherBoardConfig;

use super::ej_board_config::EjBoardConfig;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjBoard {
    pub name: String,
    pub description: String,
    pub configs: Vec<EjBoardConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjDispatcherBoard {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub configs: Vec<EjDispatcherBoardConfig>,
}
impl EjDispatcherBoard {
    pub fn from_ej_board(board: EjBoard) -> Self {
        let configs: Vec<EjDispatcherBoardConfig> = board
            .configs
            .into_iter()
            .map(|conf: EjBoardConfig| EjDispatcherBoardConfig::from_ej_board_config(conf))
            .collect();

        Self {
            id: Uuid::new_v4(),
            name: board.name,
            description: board.description,
            configs: configs,
        }
    }
}
