# =========================
# 1️⃣ Builder stage
# =========================
FROM rust:1.75-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files first (for caching)
COPY Cargo.toml Cargo.lock ./

# Create dummy src to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

RUN cargo build --release
RUN rm -rf src

# Copy full project
COPY . .

# Build real binary
RUN cargo build --release

# =========================
# 2️⃣ Runtime stage
# =========================
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime deps only
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled binary
COPY --from=builder /app/target/release/ghosthealth-guard /usr/local/bin/ghosthealth-guard

# Security: run as non-root
RUN useradd -m appuser
USER appuser

EXPOSE 3000

CMD ["ghosthealth-guard"]
