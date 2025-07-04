use std::fmt::Display;

pub struct DbConfig {
    pub database_url: String,
}

fn get_env_variable(var: &str) -> String {
    std::env::var(var).expect(&format!("Env Variable '{}' missing", var))
}
impl DbConfig {
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
