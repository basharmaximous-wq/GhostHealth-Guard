use ghosthealth_guard::blockchain::{MockBlockchainClient, BlockchainClient};

#[test]
fn mock_blockchain_returns_tx_id() {
    let client = MockBlockchainClient;
    let result = client.submit_hash("abc123").unwrap();

    assert!(result.contains("mock_tx_for"));
}
