pub mod auth_body;
pub mod error;
pub mod jwt;
pub mod prelude;
pub mod secret_hash;
pub mod sha256;

pub const ISS: &str = "EJ";
pub const AUTH_HEADER: &str = "Authorization";
pub const AUTH_HEADER_PREFIX: &str = "Bearer ";
pub const CONNECTION_TOKEN_TYPE: &str = "Bearer";
