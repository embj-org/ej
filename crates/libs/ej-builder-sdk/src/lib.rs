//! Builder SDK for the EJ framework.
//!
//! Provides communication interface between builders and the EJ dispatcher.
//!
//! # Usage
//!
//! ```rust, no_run
//! use ej_builder_sdk::{BuilderSdk, BuilderEvent};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let sdk = BuilderSdk::init(|sdk, event| async move {
//!         match event {
//!             BuilderEvent::Exit => {
//!                 // Cleanup logic here
//!                 println!("Received exit signal for: ");
//!                 println!("{} {} ({:?})", sdk.board_name(), sdk.board_config_name(), sdk.action());
//!                 std::process::exit(0);
//!             }
//!         }
//!     }).await.unwrap();
//!     
//!     // Builder logic here
//!     Ok(())
//! }
//! ```

use std::{env::args, path::PathBuf};

use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        UnixStream,
        unix::{OwnedReadHalf, OwnedWriteHalf},
    },
    signal::unix::{SignalKind, signal},
};
use tracing::info;

use crate::prelude::*;
pub mod error;
pub mod prelude;

/// Events sent from the dispatcher to the builder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuilderEvent {
    /// Request to exit the builder.
    Exit,
}

/// Responses sent from the builder to the dispatcher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuilderResponse {
    /// Acknowledge receipt of an event.
    Ack,
}
#[derive(Debug, Clone, Copy)]
pub enum Action {
    Build,
    Run,
}

impl TryFrom<&str> for Action {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        if value == "build" {
            return Ok(Action::Build);
        }
        if value == "run" {
            return Ok(Action::Run);
        }
        Err(Error::InvalidAction(String::from(value)))
    }
}

impl From<Action> for &'static str {
    fn from(value: Action) -> Self {
        match value {
            Action::Build => "build",
            Action::Run => "run",
        }
    }
}

impl From<Action> for String {
    fn from(value: Action) -> Self {
        let value: &str = value.into();
        Self::from(value)
    }
}

/// Builder SDK for communicating with the EJ dispatcher.
///
/// Handles Unix socket communication and event processing between
/// the builder and dispatcher.
#[derive(Debug, Clone)]
pub struct BuilderSdk {
    /// The board name.
    board_name: String,
    /// The board configuration name.
    board_config_name: String,
    /// The path to the config.toml file.
    config_path: String,
    /// The action the script should take.
    action: Action,
}

impl BuilderSdk {
    /// Initialize the builder SDK and start event processing.
    ///
    /// Sets up Unix socket communication with the dispatcher and starts
    /// an async event loop to handle incoming events.
    ///
    /// # Arguments
    ///
    /// * `event_callback` - Function called when events are received
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ej_builder_sdk::{BuilderSdk, BuilderEvent};
    /// # tokio_test::block_on(async {
    /// let sdk = BuilderSdk::init(|sdk, event| async move {
    ///     println!("{:?} {} {} ({:?})", event, sdk.board_name(), sdk.board_config_name(), sdk.action());
    ///     match event {
    ///         BuilderEvent::Exit => std::process::exit(0),
    ///     }
    /// }).await.unwrap();
    /// # });
    /// ```
    pub async fn init<F, Fut>(event_callback: F) -> Result<Self>
    where
        F: Fn(Self, BuilderEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let args: Vec<String> = std::env::args().into_iter().collect();
        if args.len() < 6 {
            return Err(Error::MissingArgs(6, args.len()));
        }

        let action: Action = TryFrom::<&str>::try_from(&args[1])?;

        let stream = UnixStream::connect(&args[5]).await?;
        let sdk = Self {
            config_path: args[2].clone(),
            board_name: args[3].clone(),
            board_config_name: args[4].clone(),
            action,
        };
        let sdk_loop = sdk.clone();
        let mut sigint = signal(SignalKind::interrupt())?;
        tokio::spawn(async move {
            while sigint.recv().await.is_some() {
                info!("SIGINT received");
            }
        });

        tokio::spawn(async move { sdk_loop.start_event_loop(stream, event_callback).await });
        Ok(sdk)
    }
    /// Get the action this script should take
    pub fn action(&self) -> Action {
        self.action
    }
    /// Get the path to the config.toml file.
    pub fn config_path(&self) -> PathBuf {
        PathBuf::from(&self.config_path)
    }
    /// Get the board name.
    pub fn board_name(&self) -> &str {
        &self.board_name
    }
    /// Get the board configuration name.
    pub fn board_config_name(&self) -> &str {
        &self.board_config_name
    }
    /// Parse event data from JSON string.
    fn parse_event(payload: &str) -> Result<BuilderEvent> {
        Ok(serde_json::from_str(payload)?)
    }
    /// Start the event loop for processing dispatcher messages.
    async fn start_event_loop<F, Fut>(self, stream: UnixStream, cb: F) -> Result<()>
    where
        F: Fn(Self, BuilderEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let mut payload = String::new();
        let (mut rx, mut tx) = stream.into_split();

        loop {
            tokio::select! {
                read_result = rx.read_to_string(&mut payload)  => {
                    match read_result {
                        Ok(0) => break,
                        Ok(n) => {
                            let event = BuilderSdk::parse_event(&payload)?;
                            info!("Received event from builder {:?}", event);
                            cb(self.clone(), event).await;
                            info!("Acking event to builder");
                            let response = serde_json::to_string(&BuilderResponse::Ack)?;
                            tx.write_all(response.as_bytes()).await;
                            tx.write_all(b"\n").await;
                            tx.flush().await;
                        }
                        Err(e) => return Err(Error::from(e)),
                    }
                }

                _ = tokio::signal::ctrl_c() => {
                    info!("Received Ctrl+C, shutting down...");
                    cb(self.clone(), BuilderEvent::Exit).await; // call callback with shutdown event
                    break;
                }
            }
        }
        Ok(())
    }
}
