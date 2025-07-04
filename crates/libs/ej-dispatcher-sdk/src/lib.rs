use crate::{ejjob::EjJob, ejsocket_message::EjSocketClientMessage, prelude::*};
use std::{collections::HashMap, fmt, path::Path, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    net::UnixStream,
};
use tracing::{error, info};
use uuid::Uuid;

pub mod build;
pub mod ejbuilder;
pub mod ejclient;
pub mod ejjob;
pub mod ejsocket_message;
pub mod ejws_message;
pub mod error;
pub mod prelude;
pub mod run;

async fn dispatch(stream: &mut UnixStream, job: EjJob, max_duration: Duration) -> Result<()> {
    let message = EjSocketClientMessage::Dispatch {
        job,
        timeout: max_duration,
    };

    let payload = serde_json::to_string(&message)?;
    stream.write_all(payload.as_bytes()).await;
    stream.write_all(b"\n").await;
    stream.flush().await;
    Ok(())
}
