
use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct AdmissionState {
    pub max_risk_score: u8,
}

#[derive(Deserialize)]
struct AdmissionReview {
    request: AdmissionRequest,
}

#[derive(Deserialize)]
struct AdmissionRequest {
    name: String,
    object: serde_json::Value,
}

#[derive(Serialize)]
struct AdmissionResponse {
    allowed: bool,
    status: Option<String>,
}

async fn validate(
    State(state): State<Arc<AdmissionState>>,
    Json(review): Json<AdmissionReview>
) -> Json<AdmissionResponse> {
    // Example: compute risk score from audit log embedded in pod annotations
    let risk_score = review.request.object["audit_risk_score"].as_u64().unwrap_or(0);

    if risk_score > state.max_risk_score as u64 {
        Json(AdmissionResponse {
            allowed: false,
            status: Some("Compliance risk too high".into()),
        })
    } else {
        Json(AdmissionResponse {
            allowed: true,
            status: None,
        })
    }
}

pub fn create_admission_router(state: Arc<AdmissionState>) -> Router {
    Router::new()
        .route("/validate", post(validate))
        .with_state(state)
}
