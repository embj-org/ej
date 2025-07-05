//! Board definition types.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ej_board_config::{EjBoardConfig, EjUserBoardConfig};

/// User-defined board configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjUserBoard {
    /// Board name.
    pub name: String,
    /// Board description.
    pub description: String,
    /// Board configurations.
    pub configs: Vec<EjUserBoardConfig>,
}

/// Internal board configuration with UUID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjBoard {
    /// Unique board identifier.
    pub id: Uuid,
    /// Board name.
    pub name: String,
    /// Board description.
    pub description: String,
    /// Board configurations.
    pub configs: Vec<EjBoardConfig>,
}
impl EjBoard {
    /// Convert user board to internal board with UUID.
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
