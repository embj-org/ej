use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use crate::{
    ejjob::{EjJobApi, EjJobQuery},
    ejsocket_message::{EjSocketClientMessage, EjSocketServerMessage},
    prelude::*,
    socket,
};
use std::path::Path;
pub async fn fetch_jobs(socket_path: &Path, commit_hash: String) -> Result<Vec<EjJobApi>> {
    let mut stream = UnixStream::connect(socket_path).await?;
    let message = EjSocketClientMessage::FetchJobs(EjJobQuery { commit_hash });
    socket::send(&mut stream, message).await?;
    let message: EjSocketServerMessage = socket::receive(&mut stream).await?;

    match message {
        EjSocketServerMessage::Jobs(jobs) => Ok(jobs),
        _ => Err(Error::UnexpectedSocketMessage(message)),
    }
}
