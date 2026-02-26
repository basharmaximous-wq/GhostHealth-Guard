pub trait BlockchainClient {
    fn submit_hash(&self, hash: &str) -> anyhow::Result<String>;
}

pub struct MockBlockchainClient;

impl BlockchainClient for MockBlockchainClient {
    fn submit_hash(&self, hash: &str) -> anyhow::Result<String> {
        Ok(format!("mock_tx_for_{}", hash))
    }
}