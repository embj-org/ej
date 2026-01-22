use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};
use uuid::Uuid;

use crate::{
    EjRunResult,
    ejjob::{EjJobApi, EjRunResultQuery},
    ejsocket_message::{EjSocketClientMessage, EjSocketServerMessage},
    prelude::*,
    socket,
};
use std::path::Path;
pub async fn fetch_run_result(socket_path: &Path, job_id: Uuid) -> Result<EjRunResult> {
    let mut stream = UnixStream::connect(socket_path).await?;
    let message = EjSocketClientMessage::FetchJobResults(EjRunResultQuery { job_id });
    socket::send(&mut stream, message).await?;
    let message = socket::receive(&mut stream).await?;

    match message {
        EjSocketServerMessage::RunResult(result) => Ok(result),
        _ => Err(Error::UnexpectedSocketMessage(message)),
    }
}
