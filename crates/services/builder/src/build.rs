use std::{
    sync::{atomic::AtomicBool, mpsc, Arc},
    thread,
};

use ej::ej_config::ej_config::EjConfig;
use lib_io::runner::{RunEvent, Runner};

use ej::prelude::*;
use tracing::{error, info};
pub fn build(config: &EjConfig) -> Result<()> {
    let board_count = config.boards.len();
    for (board_idx, board) in config.boards.iter().enumerate() {
        info!("Board {}/{}: {}", board_idx + 1, board_count, board.name);
        for (config_idx, board_config) in board.configs.iter().enumerate() {
            let (tx, rx) = mpsc::channel();
            let should_stop = Arc::new(AtomicBool::new(false));
            info!("\tConfig {}: {}", config_idx + 1, board_config.name);
            let runner = Runner::new(board_config.build_script.clone(), Vec::new());
            let handler = thread::spawn(move || runner.run(tx, should_stop));

            while let Ok(event) = rx.recv() {
                match event {
                    RunEvent::ProcessCreationFailed(err) => {
                        error!("\tFailed to create process {err}")
                    }
                    RunEvent::ProcessCreated => info!("\tBuild started"),
                    RunEvent::ProcessEnd(success) => {
                        if success {
                            info!("Run ended successfully")
                        } else {
                            error!("Run failed")
                        };
                        if !success {
                            return Err(Error::BuildError);
                        }
                    }
                    RunEvent::ProcessNewOutputLine(line) => info!("\t{}", line),
                }
            }
            let exit_status = handler.join()??;
            if !exit_status.success() {
                error!("Exit status {}", exit_status);
                return Err(Error::BuildError);
            }
        }
    }
    Ok(())
}
