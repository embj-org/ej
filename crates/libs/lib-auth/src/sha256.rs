use crate::prelude::*;
use serde::Serialize;
use sha2::{Digest, Sha256};

pub fn generate_hash(payload: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    let hash_result = hasher.finalize();
    format!("{:x}", hash_result)
}
