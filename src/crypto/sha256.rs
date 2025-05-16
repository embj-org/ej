use crate::prelude::*;
use serde::Serialize;
use sha2::{Digest, Sha256};

pub fn generate_hash<T: Serialize>(object: &T) -> Result<String> {
    let config_json = serde_json::to_string(&object)?;
    let mut hasher = Sha256::new();
    hasher.update(config_json.as_bytes());
    let hash_result = hasher.finalize();
    Ok(format!("{:x}", hash_result))
}
