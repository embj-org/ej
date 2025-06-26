use crate::{
    crypto::sha256::generate_hash,
    db::connection::DbConnection,
    ej_config::{
        db::{
            ej_board_config_tag_db::NewEjBoardConfigTag,
            ej_tag::{EjTag, NewEjTag},
        },
        ej_board::EjDispatcherBoard,
    },
    prelude::*,
};
use std::path::Path;

use log::info;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    db::{
        ej_board_config_db::NewEjBoardConfigDb,
        ej_board_db::NewEjBoardDb,
        ej_config_db::{EjConfigDb, NewEjConfigDb},
    },
    ej_board::EjBoard,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjGlobalConfig {
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjConfig {
    pub global: EjGlobalConfig,
    pub boards: Vec<EjBoard>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjDispatcherConfig {
    pub global: EjGlobalConfig,
    pub boards: Vec<EjDispatcherBoard>,
}

impl EjDispatcherConfig {
    pub fn from_config(config: EjConfig) -> Self {
        Self {
            global: config.global,
            boards: config
                .boards
                .into_iter()
                .map(|board| EjDispatcherBoard::from_ej_board(board))
                .collect(),
        }
    }
    pub fn save(self, builder_id: &Uuid, conn: &mut DbConnection) -> Result<Self> {
        let hash = generate_hash(&self)?;
        if let Ok(_) = EjConfigDb::fetch_client_config(conn, builder_id, &hash) {
            info!("Config already exists");
            return Ok(self);
        }
        info!("Config with hash {hash} not found for builder {builder_id}. Creating one...");
        let result = self.clone();
        let config = NewEjConfigDb::new(*builder_id, self.global.version, hash).save(conn)?;
        for board in self.boards {
            NewEjBoardDb::new(board.id, config.id.clone(), board.name, board.description)
                .save(conn)?;
            for board_config in board.configs {
                NewEjBoardConfigDb::new(board_config.id, board.id.clone(), board_config.name)
                    .save(conn)?;
                for tag in board_config.tags {
                    let tag_db = {
                        if let Ok(tag_db) = EjTag::fetch_by_name(conn, &tag) {
                            tag_db
                        } else {
                            match NewEjTag::new(&tag).save(conn) {
                                Ok(tag_db) => tag_db,
                                Err(err) => {
                                    tracing::error!("Failed to create tag {tag}: {err}");
                                    continue;
                                }
                            }
                        }
                    };
                    NewEjBoardConfigTag::new(board_config.id, tag_db.id);
                }
            }
        }
        Ok(result)
    }
}

impl EjConfig {
    pub fn from_file(file_path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(file_path)?;
        Ok(Self::from_toml(&contents)?)
    }
    pub fn from_toml(value: &str) -> Result<Self> {
        Ok(toml::from_str(value)?)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    pub fn deserialize() -> Result<()> {
        let content = r#"
            # Build Configuration File
            # This file defines boards and their configurations for building and running jobs
            
            # Global settings
            [global]
            version = "1.0.0"
            
            
            # Boards
            [[boards]]
            name = "Raspberry Pi 3"
            description = "Raspberry Pi 3 Model B+"
            
            [[boards.configs]]
            board = "rpi3"
            name = "Rpi3 Wayland"
            tags = ["wayland", "arm64"]
            build_script = "/home/work/wayland-app/scripts/build_rpi4_wayland.sh"
            run_script = "/home/work/wayland-app/scripts/run_rpi4_wayland.sh"
            results_path = "/home/work/wayland-app/results/results.json"
            library_path = "/home/work/wayland-app/lib"
            
            [[boards.configs]]
            board = "rpi3"
            name = "Rpi3 SDL"
            tags = ["sdl2", "arm64"]
            build_script = "/home/work/wayland-app/scripts/build_rpi4_wayland.sh"
            run_script = "/home/work/wayland-app/scripts/run_rpi4_wayland.sh"
            results_path = "/home/work/wayland-app/results/results.json"
            library_path = "/home/work/wayland-app/lib"
            
            [[boards]]
            name = "x86 PC running Fedora 41"
            description = "AMD Ryzen 5 3600 desktop with NVIDIA GTX 1650"
            
            [[boards.configs]]
            board = "x86_desktop"
            name = "Wayland build for desktop"
            tags = ["wayland", "x86_64"]
            build_script = "scripts/build_desktop_wayland.sh"
            run_script = "scripts/run_desktop_wayland.sh"
            results_path = "/var/log/tests/desktop_wayland_results.json"
            library_path = "https://github.com/yourusername/lib-desktop-wayland.git"
            
            [[boards.configs]]
            board = "x86_desktop"
            name = "X11 build for desktop"
            tags = ["x11", "x86_64"]
            build_script = "scripts/build_desktop_x11.sh"
            run_script = "scripts/run_desktop_x11.sh"
            results_path = "/var/log/tests/desktop_x11_results.json"
            library_path = "https://github.com/yourusername/lib-desktop-x11.git"
        "#;
        toml::from_str::<EjConfig>(content)?;
        Ok(())
    }
}
