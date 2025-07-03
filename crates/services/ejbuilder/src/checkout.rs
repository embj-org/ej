use ej::{
    ej_config::{ej_board_config::EjBoardConfig, ej_config::EjConfig},
    ej_job::results::api::EjRunOutput,
    prelude::*,
};
use lib_io::runner::{RunEvent, Runner};
use std::{
    collections::HashMap,
    io::stdout,
    sync::{Arc, atomic::AtomicBool, mpsc::channel},
};
use tracing::{error, info};
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
fn checkout(
    commit_hash: &str,
    remote_url: &str,
    remote_token: Option<String>,
    config: &EjBoardConfig,
    output: &mut EjRunOutput,
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

    for command in commands {
        let (tx, rx) = channel();
        let stop = Arc::new(AtomicBool::new(false));
        let runner = Runner::new(command[0], command[1..].to_vec());
        let result = runner.run(tx, stop);

        while let Ok(event) = rx.recv() {
            match event {
                RunEvent::ProcessCreationFailed(err) => {
                    error!("Failed to run command {:?} - {err}", command)
                }
                RunEvent::ProcessCreated => {
                    info!("Running {}", command.join(" "));
                }
                RunEvent::ProcessEnd(success) => {
                    if success {
                        info!("Command {:?} run successfully", command);
                    } else {
                        error!("Command {:?} failed", command);
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
            }
        }

        if let Some(result) = result {
            info!("Result for command {:?} {:?}", command, result);
        }
    }

    Ok(())
}
pub fn checkout_all(
    config: &EjConfig,
    commit_hash: &str,
    remote_url: &str,
    remote_token: Option<String>,
    output: &mut EjRunOutput,
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
            )?;
            paths.insert(&current_path, &config.id);
        }
    }

    Ok(())
}
pub fn handle_checkout(
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
    );

    dump_logs(&output, stdout())?;
    result
}
