use qdrant_client::prelude::*;

pub async fn store_embedding(vector: Vec<f32>, metadata: serde_json::Value) {
    let client = QdrantClient::from_url("http://qdrant:6333").build().unwrap();

    client.upsert_points(
        "phi_memory",
        vec![PointStruct::new(1, vector, metadata)],
        None,
    ).await.unwrap();
}
