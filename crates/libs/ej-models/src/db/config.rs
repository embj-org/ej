//! Database configuration management.

use std::fmt::Display;

/// Database connection configuration.
pub struct DbConfig {
    /// PostgreSQL database URL.
    pub database_url: String,
}

/// Get required environment variable or panic.
fn get_env_variable(var: &str) -> String {
    std::env::var(var).expect(&format!("Env Variable '{}' missing", var))
}
impl DbConfig {
    /// Create database configuration from environment variables.
    ///
    /// Reads the `DATABASE_URL` environment variable.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ej_models::db::config::DbConfig;
    ///
    /// let config = DbConfig::from_env();
    /// ```
    pub fn from_env() -> Self {
        Self {
            database_url: get_env_variable("DATABASE_URL"),
        }
    }
}
impl Display for DbConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "REDACTED")
    }
}
