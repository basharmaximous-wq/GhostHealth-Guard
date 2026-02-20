mod github;
mod scanner;
mod audit;
mod models;

use axum::{Router, routing::post, extract::State, body::Bytes, http::HeaderMap, response::IntoResponse};
use secrecy::{SecretString, ExposeSecret};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use sqlx::PgPool;
use std::sync::Arc;
use octocrab::{Octocrab, models::webhook_events::*};
use anyhow::Context;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
struct AppState {
    secret: SecretString,
    db: PgPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt().init();

    let db = PgPool::connect(&std::env::var("DATABASE_URL")?).await?;

    let state = Arc::new(AppState {
        secret: SecretString::new(std::env::var("GITHUB_WEBHOOK_SECRET")?),
        db,
    });

    let app = Router::new().route("/webhook", post(handle)).with_state(state);

    axum::serve(tokio::net::TcpListener::bind("0.0.0.0:3000").await?, app).await?;
    Ok(())
}

async fn handle(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {

    if verify(&state, &headers, &body).is_err() {
        return axum::http::StatusCode::UNAUTHORIZED;
    }

    axum::http::StatusCode::ACCEPTED
}

fn verify(state: &AppState, headers: &HeaderMap, body: &Bytes) -> Result<(), ()> {
    let sig = headers.get("X-Hub-Signature-256").ok_or(())?;
    let sig = sig.to_str().map_err(|_| ())?;
    let remote = sig.strip_prefix("sha256=").ok_or(())?;

    let mut mac = HmacSha256::new_from_slice(state.secret.expose_secret().as_bytes()).unwrap();
    mac.update(body);
    let local = mac.finalize().into_bytes();

    let remote = hex::decode(remote).map_err(|_| ())?;
    if local.ct_eq(&remote).unwrap_u8() != 1 {
        return Err(());
    }

    Ok(())
}
