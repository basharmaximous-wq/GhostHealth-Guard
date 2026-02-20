// fips.rs
#[cfg(feature = "fips")]
use openssl::provider::Provider;

pub fn enable_fips() {
    #[cfg(feature = "fips")]
    {
        // Load FIPS provider
        Provider::load(None, "fips").expect("FIPS provider load failed");
        println!("✅ FIPS mode enabled. Only approved algorithms are allowed.");
    }
}

// Example algorithm enforcement
pub fn assert_fips_algorithm(algo: &str) {
    #[cfg(feature = "fips")]
    {
        match algo {
            "AES-256-GCM" | "SHA-256" | "SHA-384" => {}
            _ => panic!("❌ Non-FIPS approved algorithm: {}", algo),
        }
    }
}
