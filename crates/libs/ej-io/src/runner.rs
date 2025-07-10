//! High-level async process runner with event handling.

use std::{
    io::{self, BufRead, Read},
    process::ExitStatus,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader},
    sync::mpsc::Sender,
    task::{self, JoinHandle},
    time::sleep,
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

/// High-level async process runner with event-driven output handling.
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
    async fn read_stream<T: AsyncRead + Unpin>(tx: Sender<RunEvent>, mut stream: T) {
        let mut buffer = [0; 1024];
        loop {
            let read_result = stream.read(&mut buffer).await;
            match read_result {
                Ok(0) => break,
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buffer[..n]);
                    let _ = tx
                        .send(RunEvent::ProcessNewOutputLine(data.to_string()))
                        .await;
                }
                Err(_) => break,
            }
        }
    }
    async fn launch_stream_reader<T>(tx: Sender<RunEvent>, stream: T) -> JoinHandle<()>
    where
        T: AsyncRead + Unpin + Send + 'static,
    {
        task::spawn(async move { Runner::read_stream(tx, stream).await })
    }

    /// Asynchronously run the process with event monitoring.
    ///
    /// Starts the process and monitors its execution asynchronously, sending events via the provided tokio channel.
    /// Reads stdout and stderr concurrently until the process finishes or is stopped.
    ///
    /// # Arguments
    ///
    /// * `tx` - Tokio async channel sender for RunEvent notifications
    /// * `should_stop` - Atomic flag to signal process termination
    ///
    /// # Returns
    ///
    /// Returns an `Option<ExitStatus>` - `None` if the process failed to start or was terminated,
    /// `Some(ExitStatus)` if the process completed normally.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ej_io::runner::{Runner, RunEvent};
    /// use std::sync::{Arc, atomic::AtomicBool};
    /// use tokio::sync::mpsc;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let runner = Runner::new("echo", vec!["Hello"]);
    ///     let (tx, mut rx) = mpsc::channel(100);
    ///     let should_stop = Arc::new(AtomicBool::new(false));
    ///
    ///     let exit_status = runner.run(tx, should_stop).await;
    /// }
    /// ```
    pub async fn run(
        &self,
        tx: Sender<RunEvent>,
        should_stop: Arc<AtomicBool>,
    ) -> Option<ExitStatus> {
        let mut process = spawn_process(&self.command, self.args.clone())
            .map_err(async |err| {
                let _ = tx
                    .send(RunEvent::ProcessCreationFailed(format!("{:?}", err)))
                    .await;
            })
            .ok()?;

        let _ = tx.send(RunEvent::ProcessCreated).await;

        // Launch all three tasks concurrently
        let stdout_task = if let Some(stdout) = process.stdout.take() {
            println!("Launching stdout reader function");
            Some(Runner::launch_stream_reader(tx.clone(), stdout))
        } else {
            println!("Failed to launch stdout reader function");
            None
        };

        let stderr_task = if let Some(stderr) = process.stderr.take() {
            println!("Launching stderr reader function");
            Some(Runner::launch_stream_reader(tx.clone(), stderr))
        } else {
            println!("Failed to launch stderr reader function");
            None
        };

        // Create a task that waits for the process to complete
        let process_task = task::spawn(async move {
            loop {
                if should_stop.load(Ordering::Relaxed) {
                    if stop_child(&mut process).await.is_ok() {
                        return capture_exit_status(&mut process).await.ok();
                    }
                    return None;
                }

                // Check process status
                match get_process_status(&mut process).await {
                    Err(_) => return None,
                    Ok(ProcessStatus::Done(status)) => return Some(status),
                    Ok(ProcessStatus::Running) => {
                        sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        });

        // Use join! to wait for ALL tasks to complete
        println!("Starting all tasks concurrently");
        let (process_result, stdout_result, stderr_result) = tokio::join!(
            process_task,
            async {
                if let Some(task) = stdout_task {
                    task.await;
                }
            },
            async {
                if let Some(task) = stderr_task {
                    task.await;
                }
            }
        );
        let exit_status = process_result.ok().flatten();
        let success = exit_status.map_or(false, |status| status.success());
        let _ = tx.send(RunEvent::ProcessEnd(success)).await;
        exit_status
    }
}

#[cfg(test)]
mod test {
    use std::{env, os};

    use tokio::{
        process::Command,
        sync::mpsc::{Receiver, channel},
    };

    use super::*;

    async fn compile_program(c_file: &str, target: &str) {
        let output = Command::new("gcc")
            .arg(c_file)
            .arg("-o")
            .arg(target)
            .output()
            .await
            .expect("Couldn't compile program");
    }
    async fn launch_program(
        target: &str,
        stop: Arc<AtomicBool>,
    ) -> (JoinHandle<Option<ExitStatus>>, Receiver<RunEvent>) {
        let runner = Runner::new_without_args(target.to_string());

        let (tx, rx) = channel(10);
        let thread_stop = stop.clone();

        (
            task::spawn(async move {
                let exit = runner.run(tx, thread_stop).await;
                assert!(exit.is_some());
                exit
            }),
            rx,
        )
    }
    async fn run_blocking_program(target: &str) {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let stop = Arc::new(AtomicBool::new(false));
        let (handler, _) = launch_program(target, stop.clone()).await;
        // Stop should kill the process no matter the condition it is in
        stop.store(true, Ordering::Relaxed);
        handler
            .await
            .expect("Couldn't join thread")
            .expect("Couldn't get child exit status");
    }
    async fn compile_and_run_blocking_program(c_file: &str, target: &str) {
        compile_program(c_file, target).await;
        run_blocking_program(target).await;
        let _ = std::fs::remove_file(target);
    }
    #[tokio::test]
    async fn test_stuck_stdin() {
        // This code blocks reading stdin forever
        let c_file = "./tests/assets/wait_stdin.c";
        let target = "./wait_stdin";
        compile_and_run_blocking_program(c_file, target).await;
    }

    #[tokio::test]
    async fn test_infinite_loop() {
        // This code does while(1)
        let c_file = "./tests/assets/infinite_loop.c";
        let target = "./infinite_loop";
        compile_and_run_blocking_program(c_file, target).await;
    }

    #[tokio::test]
    async fn test_infinite_loop_with_sig_mapped() {
        // This code enters an inifinite loop and ignores sigterm and sigint
        let c_file = "./tests/assets/infinite_loop_map_signals.c";
        let target = "./infinit_loop_map_signals";
        compile_and_run_blocking_program(c_file, target).await;
    }
    #[tokio::test]
    async fn test_infinite_loop_with_timeouts() {
        // This code loops forever and prints Hello * every second
        let c_file = "./tests/assets/infinite_loop.c";
        let target = "./infinite_loop_stdout";

        compile_program(c_file, target).await;

        let stop = Arc::new(AtomicBool::new(false));
        let (handler, mut rx) = launch_program(target, stop.clone()).await;

        // Give the program some time to start
        tokio::time::sleep(Duration::from_millis(1000)).await;

        // Wait for process creation with timeout
        let event = tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await
            .expect("To receive message before timeout")
            .expect("To have a message");

        assert_eq!(event, RunEvent::ProcessCreated);

        for i in 1..=4 {
            let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
                .await
                .expect("To receive message before timeout")
                .expect("To have a message");

            assert_eq!(
                event,
                RunEvent::ProcessNewOutputLine(format!("Hello {}\n", i))
            );
        }

        stop.store(true, Ordering::Relaxed);

        // Wait for handler to complete with timeout
        let join_result = tokio::time::timeout(Duration::from_secs(5), handler).await;

        join_result
            .expect("Timeout waiting for handler to complete")
            .expect("Couldn't join thread")
            .expect("Couldn't get child exit status");

        let _ = std::fs::remove_file(target);
    }
}
