use std::{fmt, path::Path, time::Duration};
use tracing::{error, info};

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::UnixStream,
};

use crate::{
    dispatch,
    ejjob::{EjBuildResult, EjJobUpdate},
    ejsocket_message::EjSocketServerMessage,
};
use crate::{
    ejjob::{EjJob, EjJobType},
    prelude::*,
};

pub async fn dispatch_build(
    socket_path: &Path,
    commit_hash: String,
    remote_url: String,
    remote_token: Option<String>,
    max_duration: Duration,
) -> Result<EjBuildResult> {
    let mut stream = UnixStream::connect(socket_path).await?;

    let job = EjJob {
        job_type: EjJobType::Build,
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
                        EjJobUpdate::BuildFinished(build_result) => return Ok(build_result),
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
    Err(Error::BuildError)
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
    async fn test_dispatch_build_success() {
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
                    assert_eq!(job.job_type, EjJobType::Build);
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
                job_type: EjJobType::Build,
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

            // Send BuildFinished update with results
            let build_result = EjBuildResult {
                success: true,
                logs: vec![(
                    EjBoardConfigApi {
                        id: Uuid::new_v4(),
                        name: "test_board".to_string(),
                        tags: vec!["test".to_string()],
                    },
                    "Test build log output".to_string(),
                )],
            };
            let build_finished =
                EjSocketServerMessage::JobUpdate(EjJobUpdate::BuildFinished(build_result));
            let response = serde_json::to_string(&build_finished).unwrap();
            stream.write_all(response.as_bytes()).await.unwrap();
            stream.write_all(b"\n").await.unwrap();
        });

        // Call the function under test
        let result = dispatch_build(
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
        let build_result = result.unwrap();
        assert!(build_result.success);
        assert_eq!(build_result.logs.len(), 1);
        assert_eq!(build_result.logs[0].1, "Test build log output");
    }

    #[tokio::test]
    async fn test_dispatch_build_connection_closed_early() {
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
                    assert_eq!(job.job_type, EjJobType::Build);
                    assert_eq!(job.commit_hash, "test_commit_hash");
                    assert_eq!(job.remote_url, "test_remote_url");
                    assert_eq!(timeout, Duration::from_secs(30));
                }
                _ => panic!("Expected Dispatch message"),
            }

            // Send success response with DispatchOk
            let dispatch_ok = EjSocketServerMessage::DispatchOk(EjDeployableJob {
                id: Uuid::new_v4(),
                job_type: EjJobType::Build,
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

            // Close the connection before sending BuildFinished
            drop(stream);
        });

        // Call the function under test
        let result = dispatch_build(
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
            Error::BuildError => {
                // This is expected
            }
            other => panic!("Expected BuildError, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_dispatch_build_invalid_message_format() {
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
                job_type: EjJobType::Build,
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
        let result = dispatch_build(
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
            Error::BuildError => {
                // This is expected since we never sent BuildFinished
            }
            other => panic!("Expected BuildError, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_dispatch_build_job_failure() {
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
                job_type: EjJobType::Build,
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

            // Send BuildFinished update with failure
            let build_result = EjBuildResult {
                success: false,
                logs: vec![(
                    EjBoardConfigApi {
                        id: Uuid::new_v4(),
                        name: "test_board".to_string(),
                        tags: vec!["test".to_string()],
                    },
                    "Test build log with error output".to_string(),
                )],
            };
            let build_finished =
                EjSocketServerMessage::JobUpdate(EjJobUpdate::BuildFinished(build_result));
            let response = serde_json::to_string(&build_finished).unwrap();
            stream.write_all(response.as_bytes()).await.unwrap();
            stream.write_all(b"\n").await.unwrap();
        });

        // Call the function under test
        let result = dispatch_build(
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
        let build_result = result.unwrap();
        assert!(!build_result.success); // Job failed
        assert_eq!(build_result.logs.len(), 1);
        assert_eq!(build_result.logs[0].1, "Test build log with error output");
    }
}
