use ej::{
    ej_builder::api::EjBuilderApi,
    ej_client::api::{EjClientLogin, EjClientLoginRequest, EjClientPost},
    ej_job::api::{EjJob, EjJobType},
    ej_message::{EjSocketClientMessage, EjSocketServerMessage},
    prelude::*,
};
use lib_requests::ApiClient;
use std::{path::PathBuf, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::UnixStream,
};

use crate::cli::{DispatchArgs, UserArgs};

pub async fn handle_dispatch(
    socket_path: &PathBuf,
    dispatch: DispatchArgs,
    job_type: EjJobType,
) -> Result<()> {
    println!("Dispatching job");
    let mut stream = UnixStream::connect(socket_path).await?;

    let job = EjJob {
        job_type: job_type,
        commit_hash: dispatch.commit_hash,
        remote_url: dispatch.remote_url,
        remote_token: dispatch.remote_token,
    };
    let message = EjSocketClientMessage::Dispatch {
        job,
        timeout: Duration::from_secs(dispatch.seconds),
    };

    let payload = serde_json::to_string(&message)?;
    stream.write_all(payload.as_bytes()).await;
    stream.write_all(b"\n").await;
    stream.flush().await;
    let mut reader = BufReader::new(stream);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        match serde_json::from_str::<EjSocketServerMessage>(&line) {
            Ok(message) => {
                println!("{}", message);
            }
            Err(e) => {
                eprintln!("Failed to parse message: {}", e);
                eprintln!("Raw message: {}", line);
            }
        }
    }

    println!("Connection closed by server");
    Ok(())
}
pub async fn handle_create_root_user(socket_path: &PathBuf, args: UserArgs) -> Result<()> {
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
