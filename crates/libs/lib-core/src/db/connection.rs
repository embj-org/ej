use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use diesel_migrations::embed_migrations;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use log::info;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

use super::config::DbConfig;
#[derive(Debug, Clone)]
pub struct DbConnection {
    pub pool: Pool<ConnectionManager<PgConnection>>,
}

impl DbConnection {
    pub fn new(config: &DbConfig) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(&config.database_url);
        let pool = Pool::builder()
            .build(manager)
            .expect("Couldn't establish connection with database");
        Self { pool }
    }
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
