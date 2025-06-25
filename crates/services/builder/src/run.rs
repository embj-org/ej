use ej::ej_config::ej_config::EjConfig;
use ej::prelude::*;
use ej::{ej_config::ej_board::EjBoard, ej_job::api::EjRunOutput};
use lib_io::runner::{RunEvent, Runner};
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;
use tracing::{error, info};

pub fn run(config: &EjConfig, output: &mut EjRunOutput) -> Result<()> {
    let stop = Arc::new(AtomicBool::new(false));

    let mut join_handlers = Vec::new();
    for board in config.boards.iter() {
        let board = board.clone();
        let stop = stop.clone();
        join_handlers.push(thread::spawn(move || run_all_configs(&board, stop)));
    }

    for (i, handler) in join_handlers.into_iter().enumerate() {
        match handler.join() {
            Ok(board_results) => {
                for (j, (logs, result)) in board_results {
                    let key = (i, j);
                    let board = &config.boards[i];
                    let config = &board.configs[j];
                    output.logs.insert(key, logs);

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
    board: &EjBoard,
    stop: Arc<AtomicBool>,
) -> HashMap<usize, (Vec<String>, Option<String>)> {
    let mut outputs = HashMap::new();
    for (i, board_config) in board.configs.iter().enumerate() {
        let (tx, rx) = channel();
        let runner = Runner::new(board_config.run_script.clone(), Vec::new());
        let stop = stop.clone();
        let join_handler = thread::spawn(move || runner.run(tx, stop));

        outputs.insert(i, (Vec::new(), None));

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
                    outputs.get_mut(&i).unwrap().0.push(line);
                }
            }
        }
        match join_handler.join() {
            Ok(exit_status) => {
                if let Ok(exit_status) = exit_status {
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
                outputs.get_mut(&i).unwrap().1 = Some(run_result);
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
