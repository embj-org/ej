use ej::{
    db::{config::DbConfig, connection::DbConnection},
    prelude::*,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{api::setup_api, dispatcher::Dispatcher, socket::setup_socket};

mod api;
pub mod dispatcher;
mod socket;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = DbConnection::new(&DbConfig::from_env());
    let dispatcher = Dispatcher::new(db);
    let api_handle = setup_api(dispatcher.clone()).await?;
    let socket_handle = setup_socket(dispatcher).await?;

    tokio::select! {
        result = api_handle => {
            tracing::error!("API server stopped: {:?}", result);
        }
        result = socket_handle => {
            tracing::error!("Socket server stopped: {:?}", result);
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Shutting down");
        }
    }

    Ok(())
}
