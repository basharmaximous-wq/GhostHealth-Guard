use chrono::Utc;
use crate::hash::generate_hash;

#[derive(Clone)]
pub struct AuditEntry {
    pub timestamp: String,
    pub data_hash: String,
    pub previous_hash: String,
    pub entry_hash: String,
}

impl AuditEntry {
    pub fn new(data: &str, previous_hash: &str) -> Self {
        let timestamp = Utc::now().to_rfc3339();
        let data_hash = generate_hash(data);
        let combined = format!("{}{}{}", timestamp, data_hash, previous_hash);
        let entry_hash = generate_hash(&combined);

        Self {
            timestamp,
            data_hash,
            previous_hash: previous_hash.to_string(),
            entry_hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_chain_links_correctly() {
        let first = AuditEntry::new("doc1", "genesis");
        let second = AuditEntry::new("doc2", &first.entry_hash);

        assert_eq!(second.previous_hash, first.entry_hash);
    }
}
