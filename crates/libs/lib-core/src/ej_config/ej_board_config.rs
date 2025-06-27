use std::fmt::{self};

use crate::{
    db::connection::DbConnection, ej_config::db::ej_board_config_tag_db::EjBoardConfigTag,
    prelude::*,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ej_config::db::ej_board_config_db::EjBoardConfigDb;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjBoardConfigApi {
    pub id: Uuid,
    pub name: String,
    pub tags: Vec<String>,
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

impl EjBoardConfigApi {
    pub fn try_from_board_config_db(
        config_db: EjBoardConfigDb,
        connection: &DbConnection,
    ) -> Result<Self> {
        let tags = EjBoardConfigTag::fetch_by_board_config(config_db.id, connection)?
            .1
            .into_iter()
            .map(|tag| tag.name)
            .collect();
        Ok(Self {
            id: config_db.id,
            name: config_db.name,
            tags: tags,
        })
    }
}
impl fmt::Display for EjBoardConfigApi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {} [{}]", self.id, self.name, self.tags.join(","))
    }
}
