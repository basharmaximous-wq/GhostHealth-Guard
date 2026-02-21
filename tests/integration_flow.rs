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
