use std::{
    process::ExitStatus,
    sync::{Arc, atomic::AtomicBool, mpsc::Sender},
    thread::{self, JoinHandle},
};

use ej_io::runner::{RunEvent, Runner};

#[derive(Debug, Clone)]
pub struct SpawnRunnerArgs {
    pub script_name: String,
    pub config_name: String,
    pub config_path: String,
    pub socket_path: String,
}

impl SpawnRunnerArgs {
    fn build_runner(self) -> Runner {
        // Set arguments for child process
        // argv[1] will be the board config name so the same script can be used for every
        // config
        // argv[2] will be the config path so the process can use this to find it's workspace
        // argv[3] is the path to the socket so that he can establish a socket connection with
        // ourselfs
        Runner::new(
            self.script_name,
            vec![self.config_name, self.config_path, self.socket_path],
        )
    }
}
pub fn spawn_runner(
    args: SpawnRunnerArgs,
    tx: Sender<RunEvent>,
    stop: Arc<AtomicBool>,
) -> JoinHandle<Option<ExitStatus>> {
    let runner = args.build_runner();
    thread::spawn(move || runner.run(tx, stop))
}
