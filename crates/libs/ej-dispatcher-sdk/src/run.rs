//! Run job dispatch and management.

use std::{collections::HashMap, fmt, path::Path, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    net::UnixStream,
};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    ejjob::{EjJob, EjJobType, EjJobUpdate, EjRunResult},
    ejsocket_message::EjSocketServerMessage,
    prelude::*,
};

use crate::dispatch;
/// Dispatch a build-and-run job to the dispatcher.
///
/// Creates a build-and-run job and sends it to the dispatcher via Unix socket.
///
/// # Arguments
///
/// * `socket_path` - Path to the dispatcher Unix socket
/// * `commit_hash` - Git commit hash to build and run
/// * `remote_url` - Git repository URL
/// * `remote_token` - Optional authentication token for private repos
/// * `max_duration` - Maximum time to wait for job completion
///
/// # Examples
///
/// ```rust,no_run
/// use ej_dispatcher_sdk::dispatch_run;
/// use std::{path::Path, time::Duration};
///
/// # tokio_test::block_on(async {
/// let job_result = dispatch_run(
///     Path::new("/tmp/dispatcher.sock"),
///     "abc123".to_string(),
///     "https://github.com/user/repo.git".to_string(),
///     None,
///     Duration::from_secs(600),
/// ).await.unwrap();
///
/// println!("Run success ? {}", job_result.success);
/// println!("Run logs    {:#?}", job_result.logs);
/// println!("Run results {:#?}", job_result.results);
/// # });
/// ```
pub async fn dispatch_run(
    socket_path: &Path,
    commit_hash: String,
    remote_url: String,
    remote_token: Option<String>,
    max_duration: Duration,
) -> Result<EjRunResult> {
    let mut stream = UnixStream::connect(socket_path).await?;

    let job = EjJob {
        job_type: EjJobType::BuildAndRun,
        commit_hash: commit_hash,
        remote_url: remote_url,
        remote_token: remote_token,
    };

    let lines = dispatch(&mut stream, job, max_duration).await?;

    let mut reader = BufReader::new(stream);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        match serde_json::from_str::<EjSocketServerMessage>(&line) {
            Ok(message) => {
                info!("{}", message);
                match message {
                    EjSocketServerMessage::JobUpdate(update) => match update {
                        EjJobUpdate::RunFinished(result) => return Ok(result),
                        _ => continue,
                    },
                    _ => continue,
                }
            }
            Err(e) => {
                error!("Failed to parse message {} - {}", line, e);
            }
        }
    }
    Err(Error::RunError)
}

#[cfg(test)]
mod tests {
    use crate::ejjob::{EjDeployableJob, EjJobCancelReason};
    use crate::ejsocket_message::EjSocketClientMessage;

    use super::*;
    use ej_config::ej_board_config::EjBoardConfigApi;
    use serde_json;
    use std::collections::HashMap;
    use tempfile::NamedTempFile;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::{UnixListener, UnixStream};
    use uuid::Uuid;

    async fn create_test_socket() -> (NamedTempFile, UnixListener) {
        let temp_file = NamedTempFile::new().unwrap();
        let socket_path = temp_file.path();

        // Remove the file so we can bind to it
        std::fs::remove_file(socket_path).unwrap();

        let listener = UnixListener::bind(socket_path).unwrap();
        (temp_file, listener)
    }

    #[tokio::test]
    async fn test_dispatch_run_success() {
        // Create a temporary Unix socket
        let (temp_file, listener) = create_test_socket().await;
        let socket_path = temp_file.path();

        // Spawn a task to handle the server side
        let server_task = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();

            // Read the dispatch message
            let mut reader = BufReader::new(&mut stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();

            // Verify the message format
            let message: EjSocketClientMessage = serde_json::from_str(&line.trim()).unwrap();
            match message {
                EjSocketClientMessage::Dispatch { job, timeout } => {
                    assert_eq!(job.job_type, EjJobType::BuildAndRun);
                    assert_eq!(job.commit_hash, "test_commit_hash");
                    assert_eq!(job.remote_url, "test_remote_url");
                    assert_eq!(job.remote_token, Some("test_token".to_string()));
                    assert_eq!(timeout, Duration::from_secs(60));
                }
                _ => panic!("Expected Dispatch message"),
            }

            // Send success response with DispatchOk
            let dispatch_ok = EjSocketServerMessage::DispatchOk(EjDeployableJob {
                id: Uuid::new_v4(),
                job_type: EjJobType::BuildAndRun,
                commit_hash: "test_commit_hash".to_string(),
                remote_url: "test_remote_url".to_string(),
                remote_token: Some("test_token".to_string()),
            });
            let response = serde_json::to_string(&dispatch_ok).unwrap();
            stream.write_all(response.as_bytes()).await.unwrap();
            stream.write_all(b"\n").await.unwrap();

            // Send JobStarted update
            let job_started =
                EjSocketServerMessage::JobUpdate(EjJobUpdate::JobStarted { nb_builders: 1 });
            let response = serde_json::to_string(&job_started).unwrap();
            stream.write_all(response.as_bytes()).await.unwrap();
            stream.write_all(b"\n").await.unwrap();

            // Send RunFinished update with results
            let run_result = EjRunResult {
                success: true,
                logs: vec![(
                    EjBoardConfigApi {
                        id: Uuid::new_v4(),
                        name: "test_board".to_string(),
                        tags: vec!["test".to_string()],
                    },
                    "Test log output".to_string(),
                )],
                results: vec![(
                    EjBoardConfigApi {
                        id: Uuid::new_v4(),
                        name: "test_board".to_string(),
                        tags: vec!["test".to_string()],
                    },
                    "Test result output".to_string(),
                )],
            };
            let run_finished =
                EjSocketServerMessage::JobUpdate(EjJobUpdate::RunFinished(run_result));
            let response = serde_json::to_string(&run_finished).unwrap();
            stream.write_all(response.as_bytes()).await.unwrap();
            stream.write_all(b"\n").await.unwrap();
        });

        // Call the function under test
        let result = dispatch_run(
            socket_path,
            "test_commit_hash".to_string(),
            "test_remote_url".to_string(),
            Some("test_token".to_string()),
            Duration::from_secs(60),
        )
        .await;

        // Wait for server task to complete
        server_task.await.unwrap();

        // Verify the result
        assert!(result.is_ok());
        let run_result = result.unwrap();
        assert!(run_result.success);
        assert_eq!(run_result.logs.len(), 1);
        assert_eq!(run_result.results.len(), 1);
        assert_eq!(run_result.logs[0].1, "Test log output");
        assert_eq!(run_result.results[0].1, "Test result output");
    }

    #[tokio::test]
    async fn test_dispatch_run_connection_closed_early() {
        // Create a temporary Unix socket
        let (temp_file, listener) = create_test_socket().await;
        let socket_path = temp_file.path();

        // Spawn a task to handle the server side - close connection immediately
        let server_task = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();

            // Read the dispatch message
            let mut reader = BufReader::new(&mut stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();

            // Verify the message format
            let message: EjSocketClientMessage = serde_json::from_str(&line.trim()).unwrap();
            match message {
                EjSocketClientMessage::Dispatch { job, timeout } => {
                    assert_eq!(job.job_type, EjJobType::BuildAndRun);
                    assert_eq!(job.commit_hash, "test_commit_hash");
                    assert_eq!(job.remote_url, "test_remote_url");
                    assert_eq!(timeout, Duration::from_secs(30));
                }
                _ => panic!("Expected Dispatch message"),
            }

            // Send success response with DispatchOk
            let dispatch_ok = EjSocketServerMessage::DispatchOk(EjDeployableJob {
                id: Uuid::new_v4(),
                job_type: EjJobType::BuildAndRun,
                commit_hash: "test_commit_hash".to_string(),
                remote_url: "test_remote_url".to_string(),
                remote_token: None,
            });
            let response = serde_json::to_string(&dispatch_ok).unwrap();
            stream.write_all(response.as_bytes()).await.unwrap();
            stream.write_all(b"\n").await.unwrap();

            // Send JobStarted update
            let job_started =
                EjSocketServerMessage::JobUpdate(EjJobUpdate::JobStarted { nb_builders: 1 });
            let response = serde_json::to_string(&job_started).unwrap();
            stream.write_all(response.as_bytes()).await.unwrap();
            stream.write_all(b"\n").await.unwrap();

            // Close the connection before sending RunFinished
            drop(stream);
        });

        // Call the function under test
        let result = dispatch_run(
            socket_path,
            "test_commit_hash".to_string(),
            "test_remote_url".to_string(),
            None,
            Duration::from_secs(30),
        )
        .await;

        // Wait for server task to complete
        server_task.await.unwrap();

        // Verify the result is an error
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::RunError => {
                // This is expected
            }
            other => panic!("Expected RunError, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_dispatch_run_invalid_message_format() {
        // Create a temporary Unix socket
        let (temp_file, listener) = create_test_socket().await;
        let socket_path = temp_file.path();

        // Spawn a task to handle the server side
        let server_task = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();

            // Read the dispatch message
            let mut reader = BufReader::new(&mut stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();

            // Send success response with DispatchOk
            let dispatch_ok = EjSocketServerMessage::DispatchOk(EjDeployableJob {
                id: Uuid::new_v4(),
                job_type: EjJobType::BuildAndRun,
                commit_hash: "test_commit_hash".to_string(),
                remote_url: "test_remote_url".to_string(),
                remote_token: None,
            });
            let response = serde_json::to_string(&dispatch_ok).unwrap();
            stream.write_all(response.as_bytes()).await.unwrap();
            stream.write_all(b"\n").await.unwrap();

            // Send JobStarted update
            let job_started =
                EjSocketServerMessage::JobUpdate(EjJobUpdate::JobStarted { nb_builders: 1 });
            let response = serde_json::to_string(&job_started).unwrap();
            stream.write_all(response.as_bytes()).await.unwrap();
            stream.write_all(b"\n").await.unwrap();

            // Send invalid JSON
            stream.write_all(b"invalid json message\n").await.unwrap();

            // Send some more valid messages to ensure we continue processing
            let job_cancelled = EjSocketServerMessage::JobUpdate(EjJobUpdate::JobCancelled(
                EjJobCancelReason::Timeout,
            ));
            let response = serde_json::to_string(&job_cancelled).unwrap();
            stream.write_all(response.as_bytes()).await.unwrap();
            stream.write_all(b"\n").await.unwrap();

            // Close the connection
            drop(stream);
        });

        // Call the function under test
        let result = dispatch_run(
            socket_path,
            "test_commit_hash".to_string(),
            "test_remote_url".to_string(),
            None,
            Duration::from_secs(30),
        )
        .await;

        // Wait for server task to complete
        server_task.await.unwrap();

        // Verify the result is an error (should continue processing and eventually timeout/close)
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::RunError => {
                // This is expected since we never sent RunFinished
            }
            other => panic!("Expected RunError, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_dispatch_run_job_failure() {
        // Create a temporary Unix socket
        let (temp_file, listener) = create_test_socket().await;
        let socket_path = temp_file.path();

        // Spawn a task to handle the server side
        let server_task = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();

            // Read the dispatch message
            let mut reader = BufReader::new(&mut stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();

            // Send success response with DispatchOk
            let dispatch_ok = EjSocketServerMessage::DispatchOk(EjDeployableJob {
                id: Uuid::new_v4(),
                job_type: EjJobType::BuildAndRun,
                commit_hash: "test_commit_hash".to_string(),
                remote_url: "test_remote_url".to_string(),
                remote_token: None,
            });
            let response = serde_json::to_string(&dispatch_ok).unwrap();
            stream.write_all(response.as_bytes()).await.unwrap();
            stream.write_all(b"\n").await.unwrap();

            // Send JobStarted update
            let job_started =
                EjSocketServerMessage::JobUpdate(EjJobUpdate::JobStarted { nb_builders: 1 });
            let response = serde_json::to_string(&job_started).unwrap();
            stream.write_all(response.as_bytes()).await.unwrap();
            stream.write_all(b"\n").await.unwrap();

            // Send RunFinished update with failure
            let run_result = EjRunResult {
                success: false,
                logs: vec![(
                    EjBoardConfigApi {
                        id: Uuid::new_v4(),
                        name: "test_board".to_string(),
                        tags: vec!["test".to_string()],
                    },
                    "Test log with error output".to_string(),
                )],
                results: vec![],
            };
            let run_finished =
                EjSocketServerMessage::JobUpdate(EjJobUpdate::RunFinished(run_result));
            let response = serde_json::to_string(&run_finished).unwrap();
            stream.write_all(response.as_bytes()).await.unwrap();
            stream.write_all(b"\n").await.unwrap();
        });

        // Call the function under test
        let result = dispatch_run(
            socket_path,
            "test_commit_hash".to_string(),
            "test_remote_url".to_string(),
            None,
            Duration::from_secs(30),
        )
        .await;

        // Wait for server task to complete
        server_task.await.unwrap();

        // Verify the result
        assert!(result.is_ok());
        let run_result = result.unwrap();
        assert!(!run_result.success); // Job failed
        assert_eq!(run_result.logs.len(), 1);
        assert_eq!(run_result.results.len(), 0);
        assert_eq!(run_result.logs[0].1, "Test log with error output");
    }
}
