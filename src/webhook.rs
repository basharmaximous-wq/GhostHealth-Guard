use axum::{extract::Json, http::HeaderMap};
use serde_json::Value;

pub async fn handle_webhook(
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> String {

    println!("Webhook received!");

    // Example: detect push event
    if let Some(event) = headers.get("X-GitHub-Event") {
        println!("Event: {:?}", event);
    }

    println!("Payload: {:?}", payload);

    "OK".to_string()
}
