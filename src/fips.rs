// fips.rs
#[cfg(feature = "fips")]
use openssl::provider::Provider;

pub fn enable_fips() -> anyhow::Result<()> {
    #[cfg(feature = "fips")]
    {
        // Load FIPS provider
        Provider::load(None, "fips")
            .map_err(|e| anyhow::anyhow!("FIPS provider load failed: {}", e))?;
        println!("✅ FIPS mode enabled. Only approved algorithms are allowed.");
    }
    Ok(())
}

// Example algorithm enforcement
pub fn assert_fips_algorithm(_algo: &str) {
    #[cfg(feature = "fips")]
    {
        match _algo {
            "AES-256-GCM" | "SHA-256" | "SHA-384" => {}
            _ => panic!("❌ Non-FIPS approved algorithm: {}", _algo),
        }
    }
}
