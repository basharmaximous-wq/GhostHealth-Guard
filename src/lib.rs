// Core security logic
pub mod scanner;
pub mod audit;
pub mod models;

// Cryptography & integrity
pub mod hash;
pub mod zk;
pub mod fips;

// External integrations
pub mod github;
pub mod blockchain;

// Domain logic
pub mod patient_processor;
pub mod remediation;

// Re-exports (clean public API surface)
pub use audit::*;
pub use blockchain::*;
pub use github::*;
pub use hash::*;
pub use models::*;
pub use scanner::*;
