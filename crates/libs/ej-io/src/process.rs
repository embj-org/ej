//! Low-level process management utilities.

use std::{
    ffi::OsStr,
    io,
    process::{Child, Command, ExitStatus, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::sleep,
    time::Duration,
};

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
/// Spawn a new process with piped stdout and stderr.
///
/// Launches a subprocess with the given command and arguments.
/// Both stdout and stderr are piped and can be accessed via the returned Child.
///
/// # Arguments
///
/// * `cmd` - Command to execute
/// * `args` - Command line arguments
///
/// # Examples
///
/// ```rust
/// use ej_io::process::spawn_process;
///
/// let mut child = spawn_process("echo", vec!["Hello".to_string()]).unwrap();
/// let output = child.stdout.take().unwrap();
/// ```
pub fn spawn_process(cmd: &str, args: Vec<String>) -> Result<Child, io::Error> {
    Command::new(OsStr::new(&cmd))
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}
/// Check process status without blocking.
///
/// Polls the process status in a non-blocking manner. Includes a small sleep
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
/// # Examples
///
/// ```rust
/// use ej_io::process::{spawn_process, get_process_status, ProcessStatus};
///
/// let mut child = spawn_process("sleep", vec!["1".to_string()]).unwrap();
///
/// loop {
///     match get_process_status(&mut child).unwrap() {
///         ProcessStatus::Done(exit_status) => {
///             println!("Process finished with: {:?}", exit_status);
///             break;
///         }
///         ProcessStatus::Running => {
///             println!("Still running...");
///         }
///     }
/// }
/// ```
pub fn get_process_status(child: &mut Child) -> Result<ProcessStatus, ProcessError> {
    match child.try_wait() {
        Ok(status) => match status {
            Some(exit_status) => Ok(ProcessStatus::Done(exit_status)),
            None => {
                sleep(Duration::from_millis(10));
                Ok(ProcessStatus::Running)
            }
        },
        Err(_) => return Err(ProcessError::WaitChildFail),
    }
}

/// Terminate a child process.
///
/// Sends a kill signal to the child process.
///
/// # Examples
///
/// ```rust
/// use ej_io::process::{spawn_process, stop_child};
///
/// let mut child = spawn_process("sleep", vec!["60".to_string()]).unwrap();
/// stop_child(&mut child).unwrap();
/// ```
pub fn stop_child(child: &mut Child) -> Result<(), io::Error> {
    child.kill()
}
/// Capture the exit status of a child process.
///
/// Waits for the child process to complete and returns its exit status.
/// This will close the stdin pipe, which can unblock processes waiting for input.
///
/// # Examples
///
/// ```rust
/// use ej_io::process::{spawn_process, capture_exit_status};
///
/// let mut child = spawn_process("echo", vec!["done".to_string()]).unwrap();
/// let exit_status = capture_exit_status(&mut child).unwrap();
/// assert!(exit_status.success());
/// ```
pub fn capture_exit_status(child: &mut Child) -> Result<ExitStatus, io::Error> {
    child.wait()
}

/// Wait for a child process with cancellation support.
///
/// Waits for the child process to complete while periodically checking
/// if it should be cancelled via the atomic boolean flag.
///
/// # Arguments
///
/// * `child` - Mutable reference to the child process
/// * `should_stop` - Atomic flag to signal process termination
///
/// # Examples
///
/// ```rust
/// use ej_io::process::{spawn_process, wait_child};
/// use std::sync::{Arc, atomic::AtomicBool};
///
/// let mut child = spawn_process("sleep", vec!["1".to_string()]).unwrap();
/// let should_stop = Arc::new(AtomicBool::new(false));
///
/// let exit_status = wait_child(&mut child, should_stop).unwrap();
/// assert!(exit_status.success());
/// ```
pub fn wait_child(
    child: &mut Child,
    should_stop: Arc<AtomicBool>,
) -> Result<ExitStatus, ProcessError> {
    loop {
        if should_stop.load(Ordering::Relaxed) {
            let _ = stop_child(child);
            return Err(ProcessError::Quit);
        }
        match get_process_status(child) {
            Ok(status) => match status {
                ProcessStatus::Done(exit_status) => return Ok(exit_status),
                ProcessStatus::Running => {
                    sleep(Duration::from_millis(10));
                }
            },
            Err(_) => {
                return Err(ProcessError::WaitChildFail);
            }
        }
    }
}
