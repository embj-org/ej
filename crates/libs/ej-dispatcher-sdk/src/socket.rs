use serde::de::DeserializeOwned;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use crate::ejsocket_message::EjSocketClientMessage;
use crate::prelude::*;

pub async fn send(stream: &mut UnixStream, message: EjSocketClientMessage) -> Result<()> {
    let payload = serde_json::to_string(&message)?;
    stream.write_all(payload.as_bytes()).await;
    stream.write_all(b"\n").await;
    stream.flush().await;
    Ok(())
}
pub async fn receive<T>(stream: &mut UnixStream) -> Result<T>
where
    T: DeserializeOwned,
{
    let mut response = String::new();
    stream.read_to_string(&mut response).await?;
    Ok(serde_json::from_str(&response)?)
}
