//! Main Crate Error

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Deserialization(#[from] toml::de::Error),

    #[error(transparent)]
    Serialization(#[from] toml::ser::Error),
}
