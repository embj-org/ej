//! Configuration error types.

/// Configuration errors.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// I/O operation failed.
    #[error(transparent)]
    IO(#[from] std::io::Error),

    /// TOML deserialization failed.
    #[error(transparent)]
    Deserialization(#[from] toml::de::Error),

    /// TOML serialization failed.
    #[error(transparent)]
    Serialization(#[from] toml::ser::Error),
}
