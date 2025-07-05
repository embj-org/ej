//! Database error types.

/// Database operation errors.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Database connection pool error.
    #[error(transparent)]
    R2D2(#[from] diesel::r2d2::PoolError),

    /// Diesel ORM operation error.
    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),
}
