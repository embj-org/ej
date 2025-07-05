//! Common types and utilities.

/// Configuration error type.
pub use crate::error::Error;

/// Configuration result type.
pub type Result<T> = core::result::Result<T, Error>;
