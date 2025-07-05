//! Board configuration types.

use crate::prelude::*;
use std::fmt::{self};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User-defined board configuration. Usually loaded from TOML files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjUserBoardConfig {
    /// Configuration name. Used to identify the configuration. Recommended to be unique.
    pub name: String,
    /// Configuration tags. Can be used for filtering results or grouping configurations.
    pub tags: Vec<String>,
    /// Build script path. The script that builds the board configuration.
    /// This script is executed before running the board.
    /// It can be used to prepare the environment, compile code, etc.
    /// The script must be executable, recommended to use the `ej-builder-sdk` crate.
    pub build_script: String,
    /// Run script path. The script that runs the board configuration.
    /// This script is executed after the build script and is used to run the program on the board.
    /// It can be used to execute the program, run tests, etc.
    /// The script must be executable, recommended to use the `ej-builder-sdk` crate.
    /// Runs are done in parallel across multiple boards, and sequentially for each board.
    pub run_script: String,
    /// Results output path. The path where the results of the run will be stored.
    /// This path is used to store the results of the run. EJ abstracts away the concept of results,
    /// meaning this can be any arbitrary data that the test produces.
    /// The results can later be retrieved from the dispatcher.
    /// Recommended to not share results paths between different boards, as this can lead to data corruption.
    pub results_path: String,
    /// Library path. This is the path that will be checked out by the builder before building the configurations.
    /// You can share this path between multiple boards.
    /// Mandatory to make this a git repository and to have the repository already setup.
    pub library_path: String,
}

/// Internal board configuration with UUID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjBoardConfig {
    /// Unique configuration identifier assigned by the system.
    pub id: Uuid,
    /// Configuration name from user input.
    pub name: String,
    /// Configuration tags from user input.
    pub tags: Vec<String>,
    /// Build script path from user input.
    pub build_script: String,
    /// Run script path from user input.
    pub run_script: String,
    /// Results output path from user input.
    pub results_path: String,
    /// Library path from user input.
    pub library_path: String,
}

/// API representation of board configuration (subset of full config).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjBoardConfigApi {
    /// Configuration identifier.
    pub id: Uuid,
    /// Configuration name.
    pub name: String,
    /// Configuration tags for filtering and identification.
    pub tags: Vec<String>,
}

impl EjBoardConfig {
    /// Convert user board config to internal config with UUID.
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

impl fmt::Display for EjBoardConfigApi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {} [{}]", self.id, self.name, self.tags.join(","))
    }
}
