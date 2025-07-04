use std::sync::{Arc, atomic::AtomicBool, mpsc};

use ej::prelude::*;
use ej::{ej_config::ej_config::EjConfig, ej_job::results::api::EjRunOutput};
use ej_io::runner::RunEvent;
use tracing::{error, info};

use crate::common::SpawnRunnerArgs;
use crate::{builder::Builder, common::spawn_runner};

pub fn build(
    builder: &Builder,
    config: &EjConfig,
    output: &mut EjRunOutput,
    stop: Arc<AtomicBool>,
) -> Result<()> {
    let board_count = config.boards.len();

    for (board_idx, board) in config.boards.iter().enumerate() {
        info!("Board {}/{}: {}", board_idx + 1, board_count, board.name);
        for (config_idx, board_config) in board.configs.iter().enumerate() {
            let (tx, rx) = mpsc::channel();
            info!("Config {}: {}", config_idx + 1, board_config.name);

            let args = SpawnRunnerArgs {
                script_name: board_config.build_script.clone(),
                config_name: board_config.name.clone(),
                config_path: builder.config_path.clone(),
                socket_path: builder.socket_path.clone(),
            };
            let stop = Arc::clone(&stop);
            let handle = spawn_runner(args, tx, stop);

            while let Ok(event) = rx.recv() {
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
                .join()
                .map_err(|err| Error::Generic(format!("Error joining build thread {:?}", err)))?
                .ok_or(Error::Generic(format!("Error joining build thread")))?;

            if !exit_status.success() {
                error!("Build exit status {}", exit_status);
                return Err(Error::BuildError);
            }
        }
    }
    Ok(())
}
