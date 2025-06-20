use ej::ej_config::ej_board::EjBoard;
use ej::ej_config::ej_config::EjConfig;
use ej::prelude::*;
use lib_io::runner::{RunEvent, Runner};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc};
use std::thread;
use tracing::error;

pub enum Event {
    ProcessEvent(RunEvent),
    ResultEvent(String),
}

pub fn run(config: &EjConfig) -> Result<()> {
    let mut join_handlers = Vec::new();

    let (tx, rx) = mpsc::channel();
    let stop = Arc::new(AtomicBool::new(false));

    for board in config.boards.iter() {
        let board = board.clone();
        let tx = tx.clone();
        let stop = stop.clone();

        join_handlers.push(thread::spawn(move || {
            run_all_configs(&board, tx.clone(), stop.clone())
        }));
    }

    while let Ok(event) = rx.recv() {
        match event {
            RunEvent::ProcessCreationFailed(err) => {
                println!("\tFailed to create process {err}")
            }
            RunEvent::ProcessCreated => println!("\tRun started"),
            RunEvent::ProcessEnd(success) => {
                let status = if success {
                    "ended successfully"
                } else {
                    "failed"
                };
                println!("\tRun {status}");
                if !success {
                    return Err(Error::RunError);
                }
            }
            RunEvent::ProcessNewOutputLine(line) => print!("\t{}", line),
        }
    }
    for handler in join_handlers {
        let result = handler.join();
        println!("join result {:?}", result);
    }
    Ok(())
}

fn run_all_configs(
    board: &EjBoard,
    tx: Sender<RunEvent>,
    stop: Arc<AtomicBool>,
) -> Result<Vec<String>> {
    let mut results = Vec::new();
    for board_config in board.configs.iter() {
        let runner = Runner::new(board_config.run_script.clone(), Vec::new());
        let exit_status = runner.run(tx.clone(), stop.clone()).map_err(|err| {
            error!("Run failure for config {} - {:?}", board_config.name, err);
            Error::RunError
        })?;
        if !exit_status.success() {
            return Err(Error::RunError);
        }
        let run_result = std::fs::read_to_string(board_config.results_path.clone()).unwrap();
        results.push(run_result);
    }
    Ok(results)
}
