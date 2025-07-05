//! Database connection management and migrations.

use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel_migrations::embed_migrations;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use tracing::info;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

use super::config::DbConfig;
/// Database connection pool wrapper.
#[derive(Debug, Clone)]
pub struct DbConnection {
    /// PostgreSQL connection pool.
    pub pool: Pool<ConnectionManager<PgConnection>>,
}

impl DbConnection {
    /// Create a new database connection pool.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ej_models::db::{connection::DbConnection, config::DbConfig};
    ///
    /// let config = DbConfig::from_env();
    /// let db = DbConnection::new(&config);
    /// ```
    pub fn new(config: &DbConfig) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(&config.database_url);
        let pool = Pool::builder()
            .build(manager)
            .expect("Couldn't establish connection with database");
        Self { pool }
    }
    /// Run database migrations and return configured connection.
    ///
    /// This method runs all pending database migrations before returning
    /// the connection, ensuring the database schema is up to date.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ej_models::db::{connection::DbConnection, config::DbConfig};
    ///
    /// let config = DbConfig::from_env();
    /// let db = DbConnection::new(&config).setup();
    /// ```
    pub fn setup(self) -> Self {
        info!("Running Database Migrations");
        self.pool
            .get()
            .expect("Couldn't get a connection from the pool to run migrations")
            .run_pending_migrations(MIGRATIONS)
            .expect("Failed to run database migrations");
        self
    }
}
