use diesel::prelude::*;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

use super::from_env;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
pub struct DBTestContext {
    pub conn: PgConnection,
}
impl DBTestContext {
    pub fn new(url: &str) -> Self {
        let mut conn = PgConnection::establish(&url).expect("Cannot connect to database");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run database migrations");

        Self { conn }
    }
    pub fn from_env() -> Self {
        Self::new(&from_env("DATABASE_URL"))
    }
}
impl Drop for DBTestContext {
    fn drop(&mut self) {
        self.conn
            .revert_all_migrations(MIGRATIONS)
            .expect("Failed to revert changes to db");
    }
}
