//! Main Crate Error

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    R2D2(#[from] diesel::r2d2::PoolError),

    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),
}
