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

pub struct Builder {
    pub config: EjConfig,
    pub config_path: String,
    pub socket_path: String,
    pub tx: Sender<BuilderEvent>,
}

impl Builder {
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
