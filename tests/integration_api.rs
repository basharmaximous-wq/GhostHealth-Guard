
use ghosthealth_guard::hash::generate_hash;
use ghosthealth_guard::zk::{generate_proof, verify_proof};

#[test]
fn full_document_processing_flow() {
    let document = "Important medical record";

    let hash = generate_hash(document);
    let proof = generate_proof(&hash);

    assert!(verify_proof(&hash, &proof));
}


