//! Common types and utilities.

/// Database error type.
pub use crate::error::Error;

/// Database result type.
pub type Result<T> = core::result::Result<T, Error>;
