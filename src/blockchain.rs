pub trait BlockchainClient {
    fn submit(&self, data: &str) -> Result<String, String>;
}

pub struct MockBlockchainClient;

impl BlockchainClient for MockBlockchainClient {
    fn submit(&self, _data: &str) -> Result<String, String> {
        Ok("mock_tx_id_123".to_string())
    }
}
