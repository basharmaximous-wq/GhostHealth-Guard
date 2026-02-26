use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use crate::hash::generate_hash;
use crate::zk::{generate_proof, verify_proof};
#[derive(Deserialize)]
pub struct DocumentRequest {
    pub document: String,
}

#[derive(Serialize)]
pub struct DocumentResponse {
    pub hash: String,
    pub proof: String,
    pub verified: bool,
}
async fn process_document(
    Json(payload): Json<DocumentRequest>,
) -> Json<DocumentResponse> {
    let hash = generate_hash(&payload.document);
    let proof = generate_proof(&hash);
    let verified = verify_proof(&hash, &proof);

    Json(DocumentResponse {
        hash,
        proof,
        verified,
    })
}

pub fn create_router() -> Router {
    Router::new().route("/process", post(process_document))
}
