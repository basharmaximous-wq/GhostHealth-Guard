#[cfg(feature = "sgx")]
use ring::signature::{Ed25519KeyPair, KeyPair};

#[cfg(feature = "sgx")]
pub fn enclave_sign(data: &[u8]) -> Vec<u8> {
    let seed = [0u8; 32]; // replace with secure sealed key
    let key = Ed25519KeyPair::from_seed_unchecked(&seed).unwrap();
    key.sign(data).as_ref().to_vec()
}
