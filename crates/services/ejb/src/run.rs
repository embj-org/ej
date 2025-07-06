//! Run execution functionality for the EJ Builder Service.
//!
//! Handles the run phase of job execution by executing run scripts
//! for each board configuration. The run process:
//!
//! 1. Spawns parallel execution threads for each board
//! 2. Within each board, executes configurations sequentially
//! 3. Collects runtime output, logs, and results
//! 4. Handles result file collection from specified paths
//! 5. Reports run success/failure status
//!
//! Boards run in parallel to maximize throughput, but configurations
//! within each board run sequentially. Run processes can be cancelled
//! if a stop signal is received.

use ej_builder_sdk::Action;
use ej_config::ej_board::EjBoard;
use ej_config::ej_config::EjConfig;
use ej_io::runner::RunEvent;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::channel;
use std::thread;
use tracing::{error, info};
use uuid::Uuid;

use crate::builder::Builder;
use crate::common::{SpawnRunnerArgs, spawn_runner};
use crate::prelude::*;
use crate::run_output::EjRunOutput;

/// Executes run scripts for all board configurations.
///
/// Runs the execution phase of job processing by spawning parallel threads
/// for each board and executing their run scripts.
///
/// # Arguments
///
/// * `builder` - The builder instance containing configuration and paths
/// * `config` - The EJ configuration with board definitions
/// * `output` - Output collector for logs and results
/// * `stop` - Atomic boolean for cancellation signal
///
/// # Returns
///
/// Returns `Ok(())` if all runs succeed, or the first error encountered.
pub fn run(
    builder: &Builder,
    config: &EjConfig,
    output: &mut EjRunOutput,
    stop: Arc<AtomicBool>,
) -> Result<()> {
    let mut join_handlers = Vec::new();
    for board in config.boards.iter() {
        let board = board.clone();
        let stop = stop.clone();

        let args = SpawnRunnerArgs {
            script_name: String::new(),
            action: Action::Run,
            board_name: board.name.clone(),
            config_name: String::new(),
            config_path: builder.config_path.clone(),
            socket_path: builder.socket_path.clone(),
        };
        join_handlers.push(thread::spawn(move || run_all_configs(args, &board, stop)));
    }

    for (i, handler) in join_handlers.into_iter().enumerate() {
        let board = &config.boards[i];
        match handler.join() {
            Ok(board_results) => {
                for (key, (mut logs, result)) in board_results {
                    let config = board
                        .configs
                        .iter()
                        .find(|c| c.id == key)
                        .expect("Failed to find config in map");

                    match result {
                        Some(result) => {
                            info!("Results for {} - {}", board.name, config.name);
                            info!("{}", result);
                            output.results.insert(key, result);
                        }
                        None => {
                            error!(
                                "Results for {} - {} are not available",
                                board.name, config.name
                            );
                        }
                    }
                    match output.logs.get_mut(&key) {
                        Some(entry) => {
                            entry.append(&mut logs);
                        }
                        None => {
                            output.logs.insert(key, logs);
                        }
                    }
                }
            }
            Err(err) => {
                error!(
                    "{} - Failed to join run board thread - {:?}",
                    config.boards[i].name, err
                );
                continue;
            }
        }
    }
    Ok(())
}

fn run_all_configs(
    mut args: SpawnRunnerArgs,
    board: &EjBoard,
    stop: Arc<AtomicBool>,
) -> HashMap<Uuid, (Vec<String>, Option<String>)> {
    let mut outputs = HashMap::new();
    for board_config in board.configs.iter() {
        let (tx, rx) = channel();

        args.script_name = board_config.build_script.clone();
        args.config_name = board_config.name.clone();
        let handle = spawn_runner(args.clone(), tx, Arc::clone(&stop));

        outputs.insert(board_config.id, (Vec::new(), None));

        while let Ok(event) = rx.recv() {
            match event {
                RunEvent::ProcessCreationFailed(err) => {
                    error!("{} - Failed to create process {}", board_config.name, err)
                }
                RunEvent::ProcessCreated => info!("{} - Run started", board_config.name),
                RunEvent::ProcessEnd(success) => {
                    if success {
                        info!("{} - Run ended successfully", board_config.name);
                    } else {
                        error!("{} - Run failed", board_config.name);
                    }
                }
                RunEvent::ProcessNewOutputLine(line) => {
                    outputs.get_mut(&board_config.id).unwrap().0.push(line);
                }
            }
        }
        match handle.join() {
            Ok(exit_status) => {
                if let Some(exit_status) = exit_status {
                    if !exit_status.success() {
                        error!("Process exited with {exit_status}");
                        continue;
                    }
                } else {
                    error!("Failed to run process for config {}", board_config.name);
                    continue;
                }
            }
            Err(err) => error!(
                "Failed to join run thread for config {} - {:?}",
                board_config.name, err
            ),
        }

        match std::fs::read_to_string(board_config.results_path.clone()) {
            Ok(run_result) => {
                outputs.get_mut(&board_config.id).unwrap().1 = Some(run_result);
            }
            Err(err) => {
                error!(
                    "Failed to get result for config {} - {err}",
                    board_config.name
                );
            }
        }
    }
    outputs
}
