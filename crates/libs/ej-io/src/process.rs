//! Low-level async process management utilities.

use std::{
    ffi::OsStr,
    io,
    process::{ExitStatus, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use tokio::process::{Child, Command};

/// Errors that can occur during process operations.
#[derive(Debug)]
pub enum ProcessError {
    /// Failed to wait for child process.
    WaitChildFail,
    /// Failed to spawn the process.
    SpawnProcessFail(io::Error),
    /// Process was terminated.
    Quit,
}

/// Current status of a running process.
pub enum ProcessStatus {
    /// Process has completed with exit status.
    Done(ExitStatus),
    /// Process is still running.
    Running,
}
/// Spawn a new async process with piped stdout and stderr.
///
/// Launches a subprocess with the given command and arguments using tokio.
/// Both stdout and stderr are piped and can be accessed via the returned Child.
///
/// # Arguments
///
/// * `cmd` - Command to execute
/// * `args` - Command line arguments
///
/// # Returns
///
/// Returns a `Result<Child, io::Error>` - the spawned tokio process or an error.
///
/// # Examples
///
/// ```rust
/// use ej_io::process::spawn_process;
///
/// #[tokio::main]
/// async fn main() {
///     let mut child = spawn_process("echo", vec!["Hello".to_string()]).unwrap();
///     let output = child.stdout.take().unwrap();
/// }
/// ```
pub fn spawn_process(cmd: &str, args: Vec<String>) -> Result<Child, io::Error> {
    Command::new(OsStr::new(&cmd))
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}
/// Asynchronously check process status without blocking.
///
/// Polls the process status in a non-blocking manner using tokio. Includes a small async sleep
/// to prevent excessive CPU usage when called in a loop.
///
/// Note: This function may never return `ProcessStatus::Done` if the process
/// is blocked waiting for stdin. Use `stop_child` and `capture_exit_status`
/// to handle such cases.
///
/// # Arguments
///
/// * `child` - Mutable reference to the child process
///
/// # Returns
///
/// Returns a `Result<ProcessStatus, ProcessError>` indicating the current process state.
///
/// # Examples
///
/// ```rust
/// use ej_io::process::{spawn_process, get_process_status, ProcessStatus};
///
/// #[tokio::main]
/// async fn main() {
///     let mut child = spawn_process("sleep", vec!["1".to_string()]).unwrap();
///
///     loop {
///         match get_process_status(&mut child).await.unwrap() {
///             ProcessStatus::Done(exit_status) => {
///                 println!("Process finished with: {:?}", exit_status);
///                 break;
///             }
///             ProcessStatus::Running => {
///                 println!("Still running...");
///             }
///         }
///     }
/// }
/// ```
pub async fn get_process_status(child: &mut Child) -> Result<ProcessStatus, ProcessError> {
    match child.try_wait() {
        Ok(status) => match status {
            Some(exit_status) => Ok(ProcessStatus::Done(exit_status)),
            None => {
                tokio::time::sleep(Duration::from_millis(10)).await;
                Ok(ProcessStatus::Running)
            }
        },
        Err(_) => return Err(ProcessError::WaitChildFail),
    }
}

/// Asynchronously terminate a child process.
///
/// Sends a kill signal to the child process using tokio.
///
/// # Arguments
///
/// * `child` - Mutable reference to the child process
///
/// # Returns
///
/// Returns a `Result<(), io::Error>` indicating success or failure.
///
/// # Examples
///
/// ```rust
/// use ej_io::process::{spawn_process, stop_child};
///
/// #[tokio::main]
/// async fn main() {
///     let mut child = spawn_process("sleep", vec!["60".to_string()]).unwrap();
///     stop_child(&mut child).await.unwrap();
/// }
/// ```
pub async fn stop_child(child: &mut Child) -> Result<(), io::Error> {
    child.kill().await
}
/// Asynchronously capture the exit status of a child process.
///
/// Waits for the child process to complete and returns its exit status using tokio.
/// This will close the stdin pipe, which can unblock processes waiting for input.
///
/// # Arguments
///
/// * `child` - Mutable reference to the child process
///
/// # Returns
///
/// Returns a `Result<ExitStatus, io::Error>` with the process exit status.
///
/// # Examples
///
/// ```rust
/// use ej_io::process::{spawn_process, capture_exit_status};
///
/// #[tokio::main]
/// async fn main() {
///     let mut child = spawn_process("echo", vec!["done".to_string()]).unwrap();
///     let exit_status = capture_exit_status(&mut child).await.unwrap();
///     assert!(exit_status.success());
/// }
/// ```
pub async fn capture_exit_status(child: &mut Child) -> Result<ExitStatus, io::Error> {
    child.wait().await
}

/// Asynchronously wait for a child process with cancellation support.
///
/// Waits for the child process to complete while periodically checking
/// if it should be cancelled via the atomic boolean flag. Uses tokio for async operation.
///
/// # Arguments
///
/// * `child` - Mutable reference to the child process
/// * `should_stop` - Atomic flag to signal process termination
///
/// # Returns
///
/// Returns a `Result<ExitStatus, ProcessError>` with the process exit status or error.
///
/// # Examples
///
/// ```rust
/// use ej_io::process::{spawn_process, wait_child};
/// use std::sync::{Arc, atomic::AtomicBool};
///
/// #[tokio::main]
/// async fn main() {
///     let mut child = spawn_process("sleep", vec!["1".to_string()]).unwrap();
///     let should_stop = Arc::new(AtomicBool::new(false));
///
///     let exit_status = wait_child(&mut child, should_stop).await.unwrap();
///     assert!(exit_status.success());
/// }
/// ```
pub async fn wait_child(
    child: &mut Child,
    should_stop: Arc<AtomicBool>,
) -> Result<ExitStatus, ProcessError> {
    loop {
        if should_stop.load(Ordering::Relaxed) {
            let _ = stop_child(child);
            return Err(ProcessError::Quit);
        }
        match get_process_status(child).await {
            Ok(status) => match status {
                ProcessStatus::Done(exit_status) => return Ok(exit_status),
                ProcessStatus::Running => {
                    tokio::time::sleep(Duration::from_millis(10));
                }
            },
            Err(_) => {
                return Err(ProcessError::WaitChildFail);
            }
        }
    }
}
