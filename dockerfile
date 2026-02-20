FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
WORKDIR /app
COPY --from=builder /app/target/release/app /app/ghosthealth
CMD ["./ghosthealth"]
