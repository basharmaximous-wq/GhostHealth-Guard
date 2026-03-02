pub fn generate_proof(hash: &str) -> String {
    format!("proof_for_{}", hash)
}

pub fn verify_proof(hash: &str, proof: &str) -> bool {
    proof == format!("proof_for_{}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proof_verifies_correctly() {
        let hash = "abc123";
        let proof = generate_proof(hash);
        assert!(verify_proof(hash, &proof));
    }

    #[test]
    fn tampered_proof_fails() {
        let hash = "abc123";
        assert!(!verify_proof(hash, "fake_proof"));
    }
}
