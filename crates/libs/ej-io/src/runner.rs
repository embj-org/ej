//! High-level process runner with event handling.

use std::{
    io::{self, BufRead, BufReader, Read},
    process::ExitStatus,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
    },
    thread::{self, JoinHandle, sleep},
    time::Duration,
};

use crate::process::{
    ProcessStatus, capture_exit_status, get_process_status, spawn_process, stop_child,
};

/// Events emitted during process execution.
#[derive(Debug, PartialEq)]
pub enum RunEvent {
    /// Process creation failed with error message.
    ProcessCreationFailed(String),
    /// Process was successfully created.
    ProcessCreated,
    /// Process ended (true = success, false = failure).
    ProcessEnd(bool),
    /// New output line from the process.
    ProcessNewOutputLine(String),
}

/// High-level process runner with event-driven output handling.
pub struct Runner {
    /// Command to execute.
    command: String,
    /// Command line arguments.
    args: Vec<String>,
}

impl Runner {
    /// Create a new runner with command and arguments.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ej_io::runner::Runner;
    ///
    /// let runner = Runner::new("ls", vec!["-la", "/tmp"]);
    /// ```
    pub fn new(command: impl Into<String>, args: Vec<impl Into<String>>) -> Self {
        Self {
            command: command.into(),
            args: args.into_iter().map(|a| a.into()).collect(),
        }
    }

    /// Create a new runner with just a command (no arguments).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ej_io::runner::Runner;
    ///
    /// let runner = Runner::new_without_args("pwd");
    /// ```
    pub fn new_without_args(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
        }
    }
    /// Get the full command string with arguments.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ej_io::runner::Runner;
    ///
    /// let runner = Runner::new("ls", vec!["-la"]);
    /// assert_eq!(runner.get_full_command(), "ls -la");
    /// ```
    pub fn get_full_command(&self) -> String {
        format!("{} {}", &self.command, &self.args.join(" "))
    }
    fn read_stream<T: Read>(tx: Sender<RunEvent>, mut stream: T) {
        let mut buffer = [0; 1024];
        loop {
            match stream.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buffer[..n]);
                    let _ = tx.send(RunEvent::ProcessNewOutputLine(data.to_string()));
                }
                Err(_) => break,
            }
        }
    }
    fn launch_stream_reader<T>(tx: Sender<RunEvent>, stream: T) -> JoinHandle<()>
    where
        T: Read + Send + 'static,
    {
        thread::spawn(move || Runner::read_stream(tx, stream))
    }

    /// Run the process with event monitoring.
    ///
    /// Starts the process and monitors its execution, sending events via the provided channel.
    /// Reads stdout and stderr until the process finishes or is stopped.
    ///
    /// # Arguments
    ///
    /// * `tx` - Channel sender for RunEvent notifications
    /// * `should_stop` - Atomic flag to signal process termination
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ej_io::runner::{Runner, RunEvent};
    /// use std::sync::{Arc, atomic::AtomicBool, mpsc};
    ///
    /// let runner = Runner::new("echo", vec!["Hello"]);
    /// let (tx, rx) = mpsc::channel();
    /// let should_stop = Arc::new(AtomicBool::new(false));
    ///
    /// let exit_status = runner.run(tx, should_stop);
    /// ```
    pub fn run(&self, tx: Sender<RunEvent>, should_stop: Arc<AtomicBool>) -> Option<ExitStatus> {
        let mut process = spawn_process(&self.command, self.args.clone())
            .map_err(|err| {
                let _ = tx.send(RunEvent::ProcessCreationFailed(format!("{:?}", err)));
            })
            .ok()?;

        let _ = tx.send(RunEvent::ProcessCreated);

        // Take stdout and stderr and launch a stream reader for each
        let mut stdout_thread = {
            if let Some(stdout) = process.stdout.take() {
                Some(Runner::launch_stream_reader(tx.clone(), stdout))
            } else {
                None
            }
        };
        let mut stderr_thread = {
            if let Some(stderr) = process.stderr.take() {
                Some(Runner::launch_stream_reader(tx.clone(), stderr))
            } else {
                None
            }
        };

        // Loop forever until we either get asked to stop or the process ends
        let exit_status = loop {
            if should_stop.load(Ordering::Relaxed) {
                if stop_child(&mut process).is_err() {
                    break None;
                }
                break capture_exit_status(&mut process).ok();
            }
            match get_process_status(&mut process) {
                Err(_) => break None,
                Ok(ProcessStatus::Done(status)) => break Some(status),
                Ok(ProcessStatus::Running) => {
                    sleep(Duration::from_millis(100));
                }
            };
        };

        // Join stdout and stderr threads
        if let Some(t) = stdout_thread.take() {
            let _ = t.join();
        }

        if let Some(t) = stderr_thread.take() {
            let _ = t.join();
        }

        let success = if let Some(exit_status) = exit_status {
            exit_status.success()
        } else {
            false
        };

        let _ = tx.send(RunEvent::ProcessEnd(success));
        exit_status
    }
}

#[cfg(test)]
mod test {
    use std::{
        env, os,
        process::Command,
        sync::mpsc::{Receiver, channel},
    };

    use ntest::timeout;

    use super::*;

    fn compile_program(c_file: &str, target: &str) {
        let output = Command::new("gcc")
            .arg(c_file)
            .arg("-o")
            .arg(target)
            .output()
            .expect("Couldn't compile program");
        println!("Output: {:?}", output);
    }
    fn launch_program(
        target: &str,
        stop: Arc<AtomicBool>,
    ) -> (JoinHandle<Option<ExitStatus>>, Receiver<RunEvent>) {
        let runner = Runner::new_without_args(target.to_string());

        let (tx, rx) = channel();
        let thread_stop = stop.clone();

        (
            thread::spawn(move || {
                let exit = runner.run(tx, thread_stop);
                assert!(exit.is_some());
                exit
            }),
            rx,
        )
    }
    fn run_blocking_program(target: &str) {
        sleep(Duration::from_secs(1));
        let stop = Arc::new(AtomicBool::new(false));
        let (handler, _) = launch_program(target, stop.clone());
        // Stop should kill the process no matter the condition it is in
        stop.store(true, Ordering::Relaxed);
        handler
            .join()
            .expect("Couldn't join thread")
            .expect("Couldn't get child exit status");
    }
    fn compile_and_run_blocking_program(c_file: &str, target: &str) {
        compile_program(c_file, target);
        run_blocking_program(target);
        let _ = std::fs::remove_file(target);
    }
    #[test]
    #[timeout(5000)]
    fn test_stuck_stdin() {
        // This code blocks reading stdin forever
        println!("{:?}", env::current_dir());
        let c_file = "./tests/assets/wait_stdin.c";
        let target = "./wait_stdin";
        compile_and_run_blocking_program(c_file, target);
    }

    #[test]
    #[timeout(5000)]
    fn test_infinite_loop() {
        // This code does while(1)
        let c_file = "./tests/assets/infinite_loop.c";
        let target = "./infinite_loop";
        compile_and_run_blocking_program(c_file, target);
    }

    #[test]
    #[timeout(5000)]
    fn test_infinite_loop_with_sig_mapped() {
        // This code enters an inifinite loop and ignores sigterm and sigint
        let c_file = "./tests/assets/infinite_loop_map_signals.c";
        let target = "./infinit_loop_map_signals";
        compile_and_run_blocking_program(c_file, target);
    }

    #[test]
    #[timeout(10000)]
    fn test_stdout_during_run() {
        //This code loops forever and prints Hello <i> every second
        let c_file = "./tests/assets/infinite_loop.c";
        let target = "./infinite_loop_stdout";
        compile_program(c_file, target);

        let stop = Arc::new(AtomicBool::new(false));
        let (handler, rx) = launch_program(target, stop.clone());

        //give the program some time to start
        sleep(Duration::from_millis(1000));

        assert_eq!(
            rx.recv().expect("Didn't receive data from process"),
            RunEvent::ProcessCreated
        );
        for i in 1..=4 {
            assert_eq!(
                rx.recv().expect("Didn't receive data from process"),
                RunEvent::ProcessNewOutputLine(String::from(format!("Hello {}\n", i)))
            );
        }
        stop.store(true, Ordering::Relaxed);
        handler
            .join()
            .expect("Couldn't join thread")
            .expect("Couldn't get child exit status");
        let _ = std::fs::remove_file(target);
    }
}
