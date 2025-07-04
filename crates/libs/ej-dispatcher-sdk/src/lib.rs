use ej::{
    ej_config::ej_board_config::EjBoardConfigApi,
    ej_job::{
        api::{EjJob, EjJobType, EjJobUpdate},
        results::api::EjBoardConfigId,
    },
    prelude::*,
};
use std::{collections::HashMap, fmt, path::Path, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    net::UnixStream,
};
use tracing::{error, info};
use uuid::Uuid;

use ej::ej_message::{EjSocketClientMessage, EjSocketServerMessage};

pub mod build;
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
