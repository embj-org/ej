use std::{
    sync::{Arc, atomic::AtomicBool, mpsc},
    thread,
};

use ej::{ej_config::ej_config::EjConfig, ej_job::results::api::EjRunOutput};
use lib_io::runner::{RunEvent, Runner};

use ej::prelude::*;
use tracing::{error, info};
pub fn build(config: &EjConfig, output: &mut EjRunOutput) -> Result<()> {
    let board_count = config.boards.len();
    for (board_idx, board) in config.boards.iter().enumerate() {
        info!("Board {}/{}: {}", board_idx + 1, board_count, board.name);
        for (config_idx, board_config) in board.configs.iter().enumerate() {
            let (tx, rx) = mpsc::channel();
            let should_stop = Arc::new(AtomicBool::new(false));
            info!("Config {}: {}", config_idx + 1, board_config.name);
            let runner = Runner::new(board_config.build_script.clone(), Vec::new());
            let handler = thread::spawn(move || runner.run(tx, should_stop));

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
            let exit_status = handler
                .join()
                .map_err(|err| Error::Generic(format!("Error joining build thread {:?}", err)))?
                .map_err(|_| Error::Generic(format!("Error joining build thread")))?;

            if !exit_status.success() {
                error!("Build exit status {}", exit_status);
                return Err(Error::BuildError);
            }
        }
    }
    Ok(())
}
