//! EJ Dispatcher Service (ejd)
//!
//! The main dispatcher service that coordinates job execution across multiple builders.
//! It provides:
//!
//! - **API Server**: REST API for job management and client operations
//! - **WebSocket Server**: Real-time communication with connected builders
//! - **Job Dispatcher**: Manages job queues and distributes work to available builders
//! - **Database Integration**: Persists job state and client information
//!
//! The dispatcher service acts as the central coordinator in the EJ system,
//! receiving job requests from clients and distributing them to connected builders
//! for execution.

use ej_models::db::{config::DbConfig, connection::DbConnection};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{api::setup_api, dispatcher::Dispatcher, socket::setup_socket};

use crate::prelude::*;
mod api;
mod dispatcher;
mod error;
mod prelude;
mod socket;

/// Main entry point for the EJ Dispatcher Service.
///
/// Initializes logging, sets up the database connection, and starts three
/// concurrent services: the dispatcher core, API server, and WebSocket server.
///
/// The service runs until a shutdown signal is received or one of the
/// components fails.
///
/// # Examples
///
/// The service is typically started with:
/// ```bash
/// export DATABASE_URL=postgres://user:password@localhost/ejd
/// export JWT_SECRET=your_jwt_secret
/// ejd
/// ```
///
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
    let (dispatcher, dispatcher_handle) = Dispatcher::create(db);
    let api_handle = setup_api(dispatcher.clone()).await?;
    let socket_handle = setup_socket(dispatcher).await?;

    tokio::select! {
        result = dispatcher_handle => {
            tracing::error!("Dispatcher task stopped: {:?}", result);
        }
        result = api_handle => {
            tracing::error!("API server stopped: {:?}", result);
        }
        result = socket_handle => {
            tracing::error!("Socket task stopped: {:?}", result);
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Shutting down");
        }
    }

    Ok(())
}
