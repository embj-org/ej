use std::env::args;

use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        UnixStream,
        unix::{OwnedReadHalf, OwnedWriteHalf},
    },
};
use tracing::info;

pub mod error;
pub use crate::error::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuilderEvent {
    Exit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuilderResponse {
    Ack,
}

pub struct BuilderSdk {
    /// The name of the current config being handled
    board_config_name: String,
    /// The path to the config.toml provided to the builder
    config_path: String,
}

impl BuilderSdk {
    pub async fn init<F>(event_callback: F) -> Result<Self>
    where
        F: Fn(BuilderEvent) + Send + Sync + 'static,
    {
        let args: Vec<String> = std::env::args().into_iter().collect();
        if args.len() < 4 {
            return Err(Error::MissingArgs(3, args.len()));
        }

        let stream = UnixStream::connect(&args[3]).await?;
        tokio::spawn(async move { BuilderSdk::start_event_loop(stream, event_callback) });

        Ok(Self {
            board_config_name: args[1].clone(),
            config_path: args[2].clone(),
        })
    }

    fn parse_event(payload: &str) -> Result<BuilderEvent> {
        Ok(serde_json::from_str(payload)?)
    }
    async fn start_event_loop<F>(stream: UnixStream, cb: F) -> Result<()>
    where
        F: Fn(BuilderEvent) + Send + Sync + 'static,
    {
        let mut payload = String::new();
        let (mut rx, mut tx) = stream.into_split();

        loop {
            match rx.read_to_string(&mut payload).await {
                Ok(0) => break,
                Ok(n) => {
                    let event = BuilderSdk::parse_event(&payload)?;
                    info!("Received event from builder {:?}", event);
                    cb(event);
                    info!("Acking event to builder");
                    let response = serde_json::to_string(&BuilderResponse::Ack)?;
                    tx.write_all(response.as_bytes()).await;
                    tx.write_all(b"\n").await;
                    tx.flush().await;
                }
                Err(e) => return Err(Error::from(e)),
            }
        }
        Ok(())
    }
}
