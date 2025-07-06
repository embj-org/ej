//! Common utilities and types for the EJ Builder Service.
//!
//! Provides shared functionality used across different modules,
//! including runner process management and argument handling.

use std::{
    process::ExitStatus,
    sync::{Arc, atomic::AtomicBool, mpsc::Sender},
    thread::{self, JoinHandle},
};

use ej_builder_sdk::Action;
use ej_io::runner::{RunEvent, Runner};

/// Arguments for spawning a runner process.
///
/// Contains all the necessary information to start a script execution
/// process with the proper configuration and communication channels.
#[derive(Debug, Clone)]
pub struct SpawnRunnerArgs {
    /// Name of the script to execute.
    pub script_name: String,
    /// Action the child process should take
    pub action: Action,
    /// Path to the configuration file.
    pub config_path: String,
    /// Name of the board.
    pub board_name: String,
    /// Name of the board configuration.
    pub config_name: String,
    /// Path to the Unix socket for communication.
    pub socket_path: String,
}

impl SpawnRunnerArgs {
    /// Builds a runner instance from the provided arguments.
    ///
    /// Creates a `Runner` with the script name and properly formatted
    /// command-line arguments for the child process.
    fn build_runner(self) -> Runner {
        // Set arguments for child process
        // argv[1] is the action the runner should take should be either `build` or `run`
        // argv[2] is the config (.toml) path
        // argv[3] is the board name
        // argv[4] is the board config name
        // argv[5] is the path to the socket so that he can establish a socket connection with ejb
        Runner::new(
            self.script_name,
            vec![
                String::from(self.action),
                self.config_path,
                self.board_name,
                self.config_name,
                self.socket_path,
            ],
        )
    }
}

/// Spawns a runner process in a separate thread.
///
/// Creates and starts a new runner process with the provided arguments,
/// communication channels, and cancellation support.
///
/// # Arguments
///
/// * `args` - Runner configuration and script information
/// * `tx` - Channel sender for receiving run events
/// * `stop` - Atomic boolean for graceful cancellation
///
/// # Returns
///
/// Returns a `JoinHandle` that can be used to wait for the process completion
/// and retrieve the exit status.
pub fn spawn_runner(
    args: SpawnRunnerArgs,
    tx: Sender<RunEvent>,
    stop: Arc<AtomicBool>,
) -> JoinHandle<Option<ExitStatus>> {
    let runner = args.build_runner();
    thread::spawn(move || runner.run(tx, stop))
}
