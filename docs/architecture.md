# GhostHealth Guard Architecture

GhostHealth Guard is a **full-stack compliance infrastructure platform** for Rust health-tech applications. It enforces **HIPAA, GDPR, SOC2** and integrates with GitHub, Kubernetes, SGX, FIPS, ZK proofs, and blockchain notarization.

---

## Architecture Overview

```text
+-----------------------+       +------------------------+
|   GitHub Pull Request  |-----> | GhostHealth Guard CI   |
|  (Code + PR diff)     |       |  / Workflow Runner     |
+-----------------------+       +------------------------+
                                    |
                                    v
                        +--------------------------+
                        | Rust Audit Engine        |
                        | - Static Privacy Scan    |
                        | - Procedural #[Sensitive]|
                        | - Unsafe Block Analysis  |
                        +--------------------------+
                                    |
                                    v
                       +---------------------------+
                       | Vector DB / RAG Context   |
                       | - Architectural PHI Track |
                       +---------------------------+
                                    |
                                    v
                       +---------------------------+
                       | ZK Proof Generator         |
                       | - Tamper-proof audit chain |
                       +---------------------------+
                                    |
                                    v
                       +---------------------------+
                       | Blockchain Notarization    |
                       | - Store Merkle Root        |
                       +---------------------------+
                                    |
                                    v
                      +----------------------------+
                      | Kubernetes Admission Ctrl  |
                      | - Prevent non-compliant    |
                      |   pods at runtime          |
                      +----------------------------+
```
---
Key Components
---

Axum Web Server – Receives GitHub webhook events

Octocrab – GitHub API integration

SQL Database (PostgreSQL) – Stores audit logs with hash chain

Vector DB (Qdrant/LanceDB) – Repository-wide context & PHI tracking

ZK-Proof Generator – Ensures tamper-proof audit chain

Blockchain Notary – Optional Ethereum transaction to notarize audits

Kubernetes Admission Controller – Rejects pods with high compliance risk

WASM Sandbox – In-cluster scanning for untrusted code
---
Multi-Tenant SaaS Support
---
tenants table in PostgreSQL stores per-tenant API keys

Audit logs are linked to tenants

Dashboard shows compliance metrics per tenant
