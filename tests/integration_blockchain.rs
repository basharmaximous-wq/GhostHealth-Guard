use ghosthealth_guard::blockchain::{BlockchainClient, MockBlockchainClient};

#[test]
fn mock_blockchain_returns_tx_id() {
    let client = MockBlockchainClient;
    let result = client.submit_hash("abc123");

    assert!(result
        .as_deref()
        .is_ok_and(|tx_id| tx_id.contains("mock_tx_for")));
}
