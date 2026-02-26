use axum::{extract::State, http::StatusCode};
use crate::AppState; // Adjust this to wherever your AppState struct is

pub async fn health(
    State(state): State<AppState>
) -> StatusCode {
    // We try a simple query to make sure the DB is actually responding
    match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::error!("Health check database error: {}", e);
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}