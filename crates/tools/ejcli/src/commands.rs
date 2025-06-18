use ej::{
    ej_client::api::EjClientPost, ej_job::api::EjJob, ej_message::EjSocketMessage, prelude::*,
};
use log::info;
use std::path::PathBuf;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use crate::cli::{CreateUserArgs, DispatchArgs};

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
pub async fn handle_create_user(socket_path: &PathBuf, args: CreateUserArgs) -> Result<()> {
    info!("Creating user");
    let mut stream = UnixStream::connect(socket_path).await?;

    let client = EjClientPost::from(args);
    let message = EjSocketMessage::CreateUser(client);
    let payload = serde_json::to_string(&message)?;
    stream.write_all(payload.as_bytes()).await;
    stream.write_all(b"\n").await;
    stream.flush().await;

    let mut response = String::new();
    stream.read_to_string(&mut response).await?;
    info!("Response {response}");
    Ok(())
}
