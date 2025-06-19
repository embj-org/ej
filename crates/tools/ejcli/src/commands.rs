use ej::{
    ej_builder::api::EjBuilderApi,
    ej_client::api::{EjClientLogin, EjClientLoginRequest, EjClientPost},
    ej_job::api::EjJob,
    ej_message::EjSocketMessage,
    prelude::*,
};
use lib_requests::ApiClient;
use log::info;
use std::path::PathBuf;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use crate::cli::{DispatchArgs, UserArgs};

pub async fn handle_dispatch(socket_path: &PathBuf, job: DispatchArgs) -> Result<()> {
    info!("Dispatching job");
    let mut stream = UnixStream::connect(socket_path).await?;

    let job = EjJob::from(job);
    let message = EjSocketMessage::Dispatch(job);
    let payload = serde_json::to_string(&message)?;
    stream.write_all(payload.as_bytes()).await;
    stream.write_all(b"\n").await;
    stream.flush().await;

    let mut response = String::new();
    stream.read_to_string(&mut response).await?;

    info!("Response {response}");
    Ok(())
}
pub async fn handle_create_root_user(socket_path: &PathBuf, args: UserArgs) -> Result<()> {
    info!("Creating user");
    let mut stream = UnixStream::connect(socket_path).await?;

    let name = args.username;
    let secret = args
        .password
        .unwrap_or(rpassword::prompt_password("Password > ").expect("Failed to get password"));

    let message = EjSocketMessage::CreateRootUser(EjClientPost { name, secret });
    let payload = serde_json::to_string(&message)?;
    stream.write_all(payload.as_bytes()).await;
    stream.write_all(b"\n").await;
    stream.flush().await;

    let mut response = String::new();
    stream.read_to_string(&mut response).await?;
    info!("Response {response}");
    Ok(())
}

pub async fn handle_create_builder(server: &str, args: UserArgs) -> Result<()> {
    info!("Creating builder");

    let client = ApiClient::new(format!("{server}/v1"));

    let name = args.username;
    let secret = args
        .password
        .unwrap_or(rpassword::prompt_password("Password > ").expect("Failed to get password"));
    let login_body = EjClientLoginRequest { name, secret };

    let payload = serde_json::to_string(&login_body)?;
    let login: EjClientLogin = client
        .post("login", payload)
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
