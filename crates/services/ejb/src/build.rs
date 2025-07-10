//! Build execution functionality for the EJ Builder Service.
//!
//! Handles the build phase of job execution by running build scripts
//! for each board configuration. The build process:
//!
//! 1. Iterates through all board configurations sequentially
//! 2. Executes the build script for each configuration
//! 3. Collects build output and logs
//! 4. Reports build success/failure status
//!
//! All build configurations are completed before any run phase begins.
//! Build scripts are executed sequentially to avoid resource conflicts,
//! as each build script is expected to utilize all available CPU cores.
//! Build processes can be cancelled if a stop signal is received.

use std::sync::{Arc, atomic::AtomicBool};

use ej_builder_sdk::Action;
use ej_config::ej_config::EjConfig;
use ej_io::runner::RunEvent;
use tokio::sync::mpsc::channel;
use tracing::{error, info};

use crate::common::SpawnRunnerArgs;
use crate::prelude::*;
use crate::run_output::EjRunOutput;
use crate::{builder::Builder, common::spawn_runner};

/// Executes build scripts for all board configurations.
///
/// Runs the build phase of job execution by executing build scripts
/// for each board configuration in the provided EJ config.
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
/// Returns `Ok(())` if all builds succeed, or the first error encountered.
pub async fn build(
    builder: &Builder,
    config: &EjConfig,
    output: &mut EjRunOutput<'_>,
    stop: Arc<AtomicBool>,
) -> Result<()> {
    let board_count = config.boards.len();

    for (board_idx, board) in config.boards.iter().enumerate() {
        info!("Board {}/{}: {}", board_idx + 1, board_count, board.name);
        for (config_idx, board_config) in board.configs.iter().enumerate() {
            let (tx, mut rx) = channel(10);
            info!("Config {}: {}", config_idx + 1, board_config.name);

            let args = SpawnRunnerArgs {
                script_name: board_config.build_script.clone(),
                action: Action::Build,
                board_name: board.name.clone(),
                config_name: board_config.name.clone(),
                config_path: builder.config_path.clone(),
                socket_path: builder.socket_path.clone(),
            };
            let stop = Arc::clone(&stop);
            let handle = spawn_runner(args, tx, stop);

            while let Some(event) = rx.recv().await {
                match event {
                    RunEvent::ProcessCreationFailed(err) => {
                        error!(
                            "{} - {} Failed to create build process - {err}",
                            board.name, board_config.name
                        )
                    }
                    RunEvent::ProcessCreated => {
                        info!("{} - {} Build started", board.name, board_config.name)
                    }
                    RunEvent::ProcessEnd(success) => {
                        if success {
                            info!(
                                "{} - {} Build ended successfully",
                                board.name, board_config.name
                            );
                        } else {
                            error!("{} - {} Build failed", board.name, board_config.name);
                        }
                    }
                    RunEvent::ProcessNewOutputLine(line) => {
                        let key = board_config.id;
                        match output.logs.get_mut(&key) {
                            Some(entry) => {
                                entry.push(line);
                            }
                            None => {
                                output.logs.insert(key, vec![line]);
                            }
                        }
                    }
                }
            }
            let exit_status = handle
                .await
                .map_err(|err| Error::ThreadJoin(err))?
                .ok_or(Error::ProcessExitStatusUnavailable)?;

            if !exit_status.success() {
                error!("Build exit status {}", exit_status);
                return Err(Error::BuildError);
            }
        }
    }
    Ok(())
}
