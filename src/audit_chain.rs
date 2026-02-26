use sha2::{Sha256, Digest};
use serde_json::Value;

pub fn compute_hash(prev_hash: &str, current: &Value) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prev_hash.as_bytes());
    hasher.update(current.to_string().as_bytes());
    hex::encode(hasher.finalize())
}
