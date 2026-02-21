use std::fs;

use ghosthealth_guard::hash::generate_hash;
use ghosthealth_guard::zk::{generate_proof, verify_proof};
use ghosthealth_guard::audit::AuditEntry;

#[test]
fn full_document_flow() {
    let document = "Important medical record";

    let hash = generate_hash(document);
    let proof = generate_proof(&hash);

    assert!(verify_proof(&hash, &proof));

    let audit_entry = AuditEntry::new(document, "genesis");
    assert_eq!(audit_entry.data_hash, hash);
}

#[test]
fn test_file_hashing() {
    let content = fs::read_to_string("tests/fixtures/sample_document.txt")
        .expect("Failed to read test file");

    let hash = generate_hash(&content);

    assert_eq!(hash.len(), 64);
}
