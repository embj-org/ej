//! Source code checkout functionality for the EJ Builder Service.
//!
//! Handles checking out source code from remote repositories using Git.
//! Supports both public and private repositories with token authentication.
//!
//! The checkout process:
//! 1. Collects all unique library paths from board configurations
//! 2. Performs deduplication to checkout each path only once
//! 3. Builds the appropriate Git URL (with token if needed)
//! 4. Clones the repository to the library path
//! 5. Checks out the specified commit hash
//! 6. Validates the checkout was successful

use crate::{prelude::*, run_output::EjRunOutput};
use ej_config::{ej_board_config::EjBoardConfig, ej_config::EjConfig};
use ej_io::runner::{RunEvent, Runner};
use std::{
    collections::HashMap,
    io::stdout,
    sync::{Arc, atomic::AtomicBool},
};
use tokio::sync::mpsc::channel;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::{builder::Builder, logs::dump_logs};

fn build_remote_url(remote_url: &str, remote_token: Option<String>) -> String {
    if remote_token.is_none() || remote_url.starts_with("git@") {
        return remote_url.to_string();
    }

    let url;
    let prefix = if remote_url.starts_with("https") {
        url = remote_url.strip_prefix("https://");
        "https://"
    } else {
        url = remote_url.strip_prefix("http://");
        "http://"
    };

    let url = url.expect("url should either start by git@, http:// or https://");
    /* Safe as per check above */
    let token = remote_token.unwrap();
    return format!("{}{}@{}", prefix, token, url);
}
async fn checkout(
    commit_hash: &str,
    remote_url: &str,
    remote_token: Option<String>,
    config: &EjBoardConfig,
    output: &mut EjRunOutput<'_>,
) -> Result<()> {
    info!(
        "Checking out library at {} for board {}",
        config.library_path, config.id
    );
    let remote_url = &build_remote_url(remote_url, remote_token.clone());
    let commands = vec![
        vec![
            "git",
            "-C",
            &config.library_path,
            "remote",
            "remove",
            "ejupstream",
        ],
        vec![
            "git",
            "-C",
            &config.library_path,
            "remote",
            "add",
            "ejupstream",
            remote_url,
        ],
        vec!["git", "-C", &config.library_path, "fetch", "ejupstream"],
        vec!["git", "-C", &config.library_path, "checkout", commit_hash],
    ];

    for (i, command) in commands.iter().enumerate() {
        let (tx, mut rx) = channel(10);
        let stop = Arc::new(AtomicBool::new(false));
        let runner = Runner::new(command[0], command[1..].to_vec());
        let result = tokio::spawn(async move { runner.run(tx, stop).await });

        while let Some(event) = rx.recv().await {
            match event {
                RunEvent::ProcessCreationFailed(err) => {
                    error!("Failed to run command {:?} - {err}", command)
                }
                RunEvent::ProcessEnd(success) => {
                    // First command is always to remove the remote, so we don't fail on it
                    if !success && i != 0 {
                        error!("Command {:?} failed", command);
                        return Err(Error::CheckoutError);
                    }
                }
                RunEvent::ProcessNewOutputLine(line) => {
                    let line = if let Some(ref token) = remote_token {
                        line.replace(token, "<REDACTED>")
                    } else {
                        line.clone()
                    };
                    match output.logs.get_mut(&config.id) {
                        Some(entry) => {
                            entry.push(line);
                        }
                        None => {
                            output.logs.insert(config.id, vec![line]);
                        }
                    };
                }
                _ => {}
            }
        }

        if let Ok(result) = result.await {
            info!("Result for command {:?} {:?}", command, result);
        }
    }

    Ok(())
}

/// Checks out source code for all board configurations.
///
/// Iterates through all board configurations in the EJ config and checks out
/// the source code for each unique library path. Uses deduplication to ensure
/// that each library path is only checked out once, even if multiple board
/// configurations reference the same path.
///
/// # Arguments
///
/// * `config` - The EJ configuration containing board definitions
/// * `commit_hash` - Git commit hash to check out
/// * `remote_url` - Git repository URL
/// * `remote_token` - Optional authentication token for private repositories
/// * `output` - Output collector for logs and results
pub async fn checkout_all(
    config: &EjConfig,
    commit_hash: &str,
    remote_url: &str,
    remote_token: Option<String>,
    output: &mut EjRunOutput<'_>,
) -> Result<()> {
    let mut paths: HashMap<&str, &Uuid> = HashMap::new();
    for board in config.boards.iter() {
        for config in board.configs.iter() {
            let current_path = &config.library_path;
            if let Some(id) = paths.get(current_path.as_str()) {
                info!("Already checked out library at {current_path} for board {id}");
                if let Some(logs) = output.logs.get(&id) {
                    output.logs.insert(config.id, logs.clone());
                    continue;
                }

                error!(
                    "Library was supposedly checked out before for board {id} but we were unable to get its output. Doing it again"
                );
            }
            checkout(
                commit_hash,
                remote_url,
                remote_token.clone(),
                config,
                output,
            )
            .await?;
            paths.insert(&current_path, &config.id);
        }
    }

    Ok(())
}

/// Handles the checkout command from CLI.
///
/// Checks out source code from the specified repository and commit hash,
/// then displays the checkout logs to stdout.
///
/// # Examples
///
/// ```bash
/// ejb checkout --commit-hash abc123 --remote-url https://github.com/user/repo.git
/// ejb checkout --commit-hash def456 --remote-url https://github.com/user/private.git --remote-token token123
/// ```
pub async fn handle_checkout(
    builder: &Builder,
    commit_hash: String,
    remote_url: String,
    remote_token: Option<String>,
) -> Result<()> {
    let mut output = EjRunOutput::new(&builder.config);
    let result = checkout_all(
        &builder.config,
        &commit_hash,
        &remote_url,
        remote_token,
        &mut output,
    )
    .await;

    dump_logs(&output, stdout())?;
    result
}
