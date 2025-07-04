//! SHA-256 hashing for data integrity.

use crate::prelude::*;
use serde::Serialize;
use sha2::{Digest, Sha256};

/// Generates a SHA-256 hash of the provided string.
///
/// Returns a 64-character lowercase hexadecimal string.
///
/// # Arguments
///
/// * `payload` - The string data to hash
///
/// # Examples
///
/// ```rust
/// use ej_auth::sha256::generate_hash;
///
/// let data = "Hello, world!";
/// let hash = generate_hash(data);
/// assert_eq!(hash.len(), 64);
/// ```
pub fn generate_hash(payload: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    let hash_result = hasher.finalize();
    format!("{:x}", hash_result)
}
