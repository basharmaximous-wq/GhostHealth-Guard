use sha2::{Digest, Sha256};
pub fn generate_hash(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_deterministic() {
        let h1 = generate_hash("hello");
        let h2 = generate_hash("hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_is_64_chars() {
        let h = generate_hash("test");
        assert_eq!(h.len(), 64);
    }
}
