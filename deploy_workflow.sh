#!/bin/bash
set -e

# ==========================
# ENVIRONMENT VARIABLES
# ==========================
export DATABASE_URL=postgres://postgres:ghost@localhost:5432/ghosthealth
export FIPS_MODE=true
export SGX_MODE=true
export OPENAI_API_KEY=<YOUR_KEY>
export GITHUB_APP_ID=<YOUR_APP_ID>
export PRIVATE_KEY_PATH=private-key.pem

# ==========================
# 1. Build Rust Binary
# ==========================
echo "🔨 Building Rust compliance engine..."
if [ "$SGX_MODE" = "true" ]; then
    cargo build --release --features sgx,fips
else
    cargo build --release --features fips
fi

# ==========================
# 2. Docker Build
# ==========================
echo "🐳 Building Docker image..."
docker build -t ghosthealth-guard:latest .

# ==========================
# 3. Run Database (Docker Compose)
# ==========================
echo "🗄️ Starting PostgreSQL with Docker Compose..."
docker compose up -d db

# ==========================
# 4. Start Vector DB (Qdrant)
# ==========================
docker run -d --name qdrant -p 6333:6333 qdrant/qdrant:v1.2.0

# ==========================
# 5. Run WASM Sandbox Service
# ==========================
docker run -d --name wasm-sandbox ghosthealth-guard:latest \
    --run-wasm-sandbox

# ==========================
# 6. Start Axum Admission Controller
# ==========================
docker run -d --name admission-server ghosthealth-guard:latest \
    --admission-server

# ==========================
# 7. Continuous Audit Chain with ZK Proof
# ==========================
echo "🔗 Generating initial ZK proof for audit chain..."
cargo run --release --features fips,sgx --bin zk_audit_init

# ==========================
# 8. Blockchain Notarization
# ==========================
echo "⛓️ Notarizing initial audit chain on Ethereum..."
cargo run --release --features fips,sgx --bin blockchain_notary

# ==========================
# 9. Kubernetes Deployment
# ==========================
echo "🚀 Deploying GhostHealth Guard to Kubernetes..."
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/qdrant.yaml

# ==========================
# 10. Apply Admission Webhook Configuration
# ==========================
echo "📌 Applying Admission Controller Webhook..."
kubectl apply -f k8s/admission-webhook.yaml

echo "✅ Deployment complete!"
