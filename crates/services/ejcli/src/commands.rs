use ej_dispatcher_sdk::ejbuilder::EjBuilderApi;
use ej_dispatcher_sdk::ejclient::{EjClientLogin, EjClientLoginRequest, EjClientPost};
use ej_dispatcher_sdk::ejsocket_message::{EjSocketClientMessage, EjSocketServerMessage};
use ej_dispatcher_sdk::fetch_run_result::fetch_run_result;
use ej_dispatcher_sdk::run::dispatch_run;
use ej_dispatcher_sdk::{build::dispatch_build, ejjob::EjJobType};
use ej_requests::ApiClient;
use std::cmp::Ordering;
use std::path::Path;
use std::{collections::HashMap, path::PathBuf, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::UnixStream,
};
use uuid::Uuid;

use crate::cli::{DispatchArgs, UserArgs};
use ej_dispatcher_sdk::{fetch_jobs::fetch_jobs, prelude::*};

pub async fn handle_dispatch(
    socket_path: &Path,
    dispatch: DispatchArgs,
    job_type: EjJobType,
) -> Result<()> {
    println!("Dispatching job");

    if job_type == EjJobType::Build {
        let build_result = dispatch_build(
            socket_path,
            dispatch.commit_hash,
            dispatch.remote_url,
            dispatch.remote_token,
            Duration::from_secs(dispatch.seconds),
        )
        .await?;
        println!("Received Build Result {}", build_result);
    } else {
        let run_result = dispatch_run(
            socket_path,
            dispatch.commit_hash,
            dispatch.remote_url,
            dispatch.remote_token,
            Duration::from_secs(dispatch.seconds),
        )
        .await?;
        println!("Received Run Result {}", run_result);
    }
    Ok(())
}
pub async fn handle_create_root_user(socket_path: &Path, args: UserArgs) -> Result<()> {
    println!("Creating user");
    let mut stream = UnixStream::connect(socket_path).await?;

    let name = args.username;
    let secret = args
        .password
        .unwrap_or(rpassword::prompt_password("Password > ").expect("Failed to get password"));

    let message = EjSocketClientMessage::CreateRootUser(EjClientPost { name, secret });
    let payload = serde_json::to_string(&message)?;
    stream.write_all(payload.as_bytes()).await;
    stream.write_all(b"\n").await;
    stream.flush().await;

    let mut response = String::new();
    stream.read_to_string(&mut response).await?;
    let response: EjSocketServerMessage = serde_json::from_str(&response)?;
    println!("{:?}", response);
    Ok(())
}

pub async fn handle_create_builder(server: &str, args: UserArgs) -> Result<()> {
    println!("Creating builder");

    let client = ApiClient::new(format!("{server}/v1"));

    let name = args.username;
    let secret = args
        .password
        .unwrap_or(rpassword::prompt_password("Password > ").expect("Failed to get password"));
    let login_body = EjClientLoginRequest { name, secret };

    let payload = serde_json::to_string(&login_body)?;
    let login: EjClientLogin = client
        .post_and_deserialize("login", payload)
        .await
        .expect("Failed to login");

    let builder: EjBuilderApi = client
        .post_no_body("client/builder")
        .await
        .expect("Failed to create builder");

    println!("export EJB_ID={}", builder.id);
    println!("export EJB_TOKEN={}", builder.token);

    Ok(())
}

pub async fn handle_fetch_jobs(socket: &Path, commit_hash: String) -> Result<()> {
    let mut jobs = fetch_jobs(&socket, commit_hash.clone()).await?;
    println!(
        "Found {} job(s) associated with {} commit",
        jobs.len(),
        commit_hash
    );

    jobs.sort_by(|a, b| match (&a.finished_at, &b.finished_at) {
        (Some(a_finished), Some(b_finished)) => a_finished.cmp(b_finished),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    });

    for job in jobs {
        println!("{}", job);
    }
    Ok(())
}

pub async fn handle_fetch_run_results(socket: &Path, job_id: Uuid) -> Result<()> {
    let run_result = fetch_run_result(&socket, job_id).await?;
    println!("{}", run_result);
    Ok(())
}
