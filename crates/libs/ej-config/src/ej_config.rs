//! Core configuration types for the EJ framework.

use crate::{ej_board::EjBoard, prelude::*};
use std::path::Path;

use ej_auth::sha256::generate_hash;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use super::ej_board::EjUserBoard;

/// Global configuration settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjGlobalConfig {
    /// Configuration version.
    pub version: String,
}

/// User-provided configuration from TOML files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjUserConfig {
    /// Global settings.
    pub global: EjGlobalConfig,
    /// Board definitions.
    pub boards: Vec<EjUserBoard>,
}

/// Internal configuration with generated UUIDs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjConfig {
    /// Global settings.
    pub global: EjGlobalConfig,
    /// Board definitions with UUIDs.
    pub boards: Vec<EjBoard>,
}

impl EjConfig {
    /// Convert user configuration to internal configuration.
    ///
    /// Assigns UUIDs to boards and configurations.
    pub fn from_user_config(config: EjUserConfig) -> Self {
        Self {
            global: config.global,
            boards: config
                .boards
                .into_iter()
                .map(|board| EjBoard::from_ej_board(board))
                .collect(),
        }
    }
}

impl EjUserConfig {
    /// Load configuration from a TOML file.
    pub fn from_file(file_path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(file_path)?;
        Ok(Self::from_toml(&contents)?)
    }
    /// Parse configuration from TOML string.
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
        toml::from_str::<EjUserConfig>(content)?;
        Ok(())
    }
}
