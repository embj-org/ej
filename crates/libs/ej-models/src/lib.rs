//! Database models and ORM layer for the EJ framework.
//!
//! Provides Diesel-based database models, queries, and connection management
//! for all EJ entities including clients, builders, jobs, and permissions.
//!
//! # Usage
//!
//! ```rust,no_run
//! use ej_models::{client::ejclient::EjClient, db::{config::DbConfig, connection::DbConnection}};
//!
//! // Get database connection
//! let config = DbConfig::from_env();
//! let mut conn = DbConnection::new(&config);
//!
//! // Query for clients
//! let clients = EjClient::fetch_all(&mut conn).unwrap();
//! println!("Found {} clients", clients.len());
//! ```

pub mod auth;
pub mod builder;
pub mod client;
pub mod config;
pub mod db;
pub mod error;
pub mod job;
pub mod prelude;
mod schema;
