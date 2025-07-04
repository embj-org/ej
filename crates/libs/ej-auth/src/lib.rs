pub mod auth_body;
pub mod error;
pub mod jwt;
pub mod prelude;
pub mod secret_hash;
pub mod sha256;

pub const CONNECTION_TOKEN_TYPE: &str = "Bearer";
pub const ISS: &str = "EJ";
