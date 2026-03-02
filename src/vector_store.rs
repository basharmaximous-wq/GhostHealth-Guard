use qdrant_client::prelude::*;

pub async fn store_embedding(vector: Vec<f32>, metadata: serde_json::Value) -> anyhow::Result<()> {
    let client = QdrantClient::from_url("http://qdrant:6333").build()
        .map_err(|e| anyhow::anyhow!("Failed to build Qdrant client: {}", e))?;

    client.upsert_points(
        "phi_memory",
        vec![PointStruct::new(1, vector, metadata)],
        None,
    ).await.map_err(|e| anyhow::anyhow!("Upsert failed: {}", e))?;
    Ok(())
}
