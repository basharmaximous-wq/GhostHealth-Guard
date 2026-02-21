use ghosthealth_guard::audit::AuditEntry;
use ghosthealth_guard::hash::generate_hash;

#[test]
fn audit_chain_links_properly() {
    let first = AuditEntry::new("doc1", "genesis");
    let second = AuditEntry::new("doc2", &first.entry_hash);

    assert_eq!(second.previous_hash, first.entry_hash);
}

#[test]
fn audit_hash_matches_document_hash() {
    let document = "Medical record";
    let audit = AuditEntry::new(document, "genesis");

    let expected_hash = generate_hash(document);
    assert_eq!(audit.data_hash, expected_hash);
}
