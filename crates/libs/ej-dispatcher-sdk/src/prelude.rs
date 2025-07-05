//! Common types and utilities.

/// Dispatcher SDK error type.
pub use crate::error::Error;

/// Dispatcher SDK result type.
pub type Result<T> = core::result::Result<T, Error>;

/// Generic wrapper for newtype pattern.
pub struct W<T>(pub T);
