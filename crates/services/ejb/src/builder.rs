//! Builder core functionality for the EJ Builder Service.
//!
//! Provides the main `Builder` struct that manages configuration loading
//! and local Unix socket communication for child processes. The Builder
//! sets up a Unix socket server to communicate with spawned build/run scripts.

use crate::prelude::*;
use ej_builder_sdk::BuilderEvent;
use ej_config::ej_config::{EjConfig, EjUserConfig};
use futures_util::lock::Mutex;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    io::AsyncWriteExt,
    net::UnixStream,
    sync::mpsc::{Receiver, Sender, channel},
    task::JoinHandle,
};
use tracing::{error, info, warn};

/// Core builder instance that manages configuration and local communication.
///
/// The Builder handles local Unix socket communication with child processes
/// (build and run scripts) spawned during job execution. It provides a
/// communication channel for these processes to send events and data back
/// to the main builder process.
pub struct Builder {
    /// The loaded EJ configuration.
    pub config: EjConfig,
    /// Path to the configuration file.
    pub config_path: String,
    /// Path to the Unix socket for communication.
    pub socket_path: String,
    /// Channel sender for builder events.
    pub tx: Sender<BuilderEvent>,
}

impl Builder {
    /// Creates a new builder instance.
    ///
    /// Loads the configuration from the specified path and sets up
    /// local Unix socket communication for child processes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// use ejb::builder::Builder;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config_path = PathBuf::from("config.toml");
    /// let socket_path = PathBuf::from("/tmp/ejb.sock");
    ///
    /// let builder = Builder::create(config_path, socket_path).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(config_path: PathBuf, socket_path: PathBuf) -> Result<Self> {
        let config = EjUserConfig::from_file(&config_path)?;
        let config = EjConfig::from_user_config(config);
        let (tx, rx) = channel(32);
        Builder::start_thread(rx, &socket_path).await?;
        let config_path_str = config_path
            .into_os_string()
            .into_string()
            .expect(&format!("Failed to convert config path to a valid string",));
        let socket_path_str = socket_path
            .into_os_string()
            .into_string()
            .expect("Failed to convert socket path to a valid string");

        Ok(Self {
            config,
            config_path: config_path_str,
            socket_path: socket_path_str,
            tx,
        })
    }

    async fn start_thread(
        mut rx: Receiver<BuilderEvent>,
        socket_path: &Path,
    ) -> Result<JoinHandle<()>> {
        let _ = std::fs::remove_file(&socket_path);
        let listener = tokio::net::UnixListener::bind(socket_path)?;
        let channels = Arc::new(Mutex::new(Vec::<Sender<BuilderEvent>>::new()));
        let t_channels = channels.clone();
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                info!("New Builder Message. Message {:?}", message,);
                for tx in t_channels.lock().await.iter() {
                    if let Err(err) = tx.send(message.clone()).await {
                        warn!("Failed to send message to {err}");
                    }
                }
            }
        });

        Ok(tokio::spawn(async move {
            let mut connection_count = 0;
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let id = connection_count;
                        connection_count += 1;
                        let (tx, rx) = channel(2);
                        let channels = channels.clone();
                        tokio::spawn(async move {
                            let index = {
                                let mut channels = channels.lock().await;
                                channels.push(tx);
                                channels.len() - 1
                            };
                            info!(
                                "New socket connection {id}. # Connected clients {}",
                                index + 1
                            );
                            if let Err(e) = Builder::handle_connection(stream, rx).await {
                                error!("Error handling client: {}", e);
                            }
                            info!("Socket connection {id} ended");
                            channels.lock().await.remove(index);
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept connection: {}", e);
                    }
                }
            }
        }))
    }
    async fn handle_connection(stream: UnixStream, mut rx: Receiver<BuilderEvent>) -> Result<()> {
        let (_, mut writer) = stream.into_split();

        while let Some(message) = rx.recv().await {
            let serialized_response = serde_json::to_string(&message)?;
            writer.write_all(serialized_response.as_bytes()).await?;
            writer.write_all(b"\n").await?;
        }
        Ok(())
    }
}
