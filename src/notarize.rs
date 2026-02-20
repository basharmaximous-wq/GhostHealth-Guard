use ethers::prelude::*;
use std::sync::Arc;

pub async fn notarize(root_hash: String) -> anyhow::Result<TxHash> {
    let provider = Provider::<Http>::try_from("https://rpc.ankr.com/eth")?;
    let wallet: LocalWallet = "PRIVATE_KEY".parse()?;
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);

    let tx = TransactionRequest::new()
        .to("0x0000000000000000000000000000000000000000")
        .data(root_hash.into_bytes());

    let pending = client.send_transaction(tx, None).await?;
    let receipt = pending.await?;

    Ok(receipt.transaction_hash)
}
