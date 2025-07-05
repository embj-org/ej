//! Configuration management for the EJ framework.
//!
//! Provides types and utilities for managing EJ board configurations and global settings.
//!
//! # Usage
//!
//! ```rust
//! use ej_config::{EjUserConfig, EjConfig};
//! use std::path::Path;
//!
//! // Load user configuration from TOML file
//! let user_config = EjUserConfig::from_file(Path::new("../../../examples/config.toml")).unwrap();
//!
//! // Convert to internal configuration
//! let config = EjConfig::from_user_config(user_config);
//! ```

pub mod ej_board;
pub mod ej_board_config;
pub mod ej_config;
pub mod error;
pub mod prelude;

pub use ej_config::{EjConfig, EjUserConfig};
