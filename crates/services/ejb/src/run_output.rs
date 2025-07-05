//! Output collection and management for job execution.
//!
//! Provides the `EjRunOutput` struct for collecting and organizing
//! execution results, logs, and artifacts from build and run processes.

use std::collections::HashMap;

use ej_config::ej_config::EjConfig;
use uuid::Uuid;

/// Collects and organizes output from job execution processes.
///
/// Stores logs and results indexed by configuration UUID for easy
/// retrieval and reporting back to the dispatcher.
#[derive(Debug)]
pub struct EjRunOutput<'a> {
    /// Reference to the EJ configuration.
    pub config: &'a EjConfig,
    /// Execution logs indexed by configuration ID.
    pub logs: HashMap<Uuid, Vec<String>>,
    /// Execution results indexed by configuration ID.
    pub results: HashMap<Uuid, String>,
}

impl<'a> EjRunOutput<'a> {
    /// Creates a new output collector for the given configuration.
    pub fn new(config: &'a EjConfig) -> Self {
        Self {
            config,
            logs: HashMap::new(),
            results: HashMap::new(),
        }
    }
}
