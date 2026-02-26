# GhostHealth Guard — How Everything Connects

## The Big Picture

```
Developer opens PR
       ↓
GitHub sends webhook → Your Axum server (main.rs)
       ↓
verify_webhook() — checks HMAC signature
       ↓
process_pull_request() — lib.rs
       ↓
├── AuditEntry::new() — creates audit record (audit.rs)
├── generate_hash() — hashes the document (hash.rs)
├── llm_review() — sends diff to GPT-4o (audit.rs)
├── BlockchainClient — notarizes the audit (blockchain.rs)
└── Posts comments back to GitHub PR (github.rs)
       ↓
Saves results to PostgreSQL (db.rs)
```

---

## The Files & What They Do

| File | What it does |
|------|-------------|
| `main.rs` | Entry point — starts the Axum web server, connects to DB |
| `lib.rs` | Core logic — AppState, WebhookEvent, process_pull_request |
| `audit.rs` | AuditEntry struct + llm_review() via GPT-4o |
| `hash.rs` | SHA-256 hashing for audit chain integrity |
| `blockchain.rs` | Notarizes audit records immutably |
| `github.rs` | Posts review comments back to GitHub PRs |
| `scanner.rs` | Semgrep-based static PHI scanning |
| `models.rs` | Shared data types (AuditResult etc.) |
| `webhook.rs` | Webhook parsing and routing |
| `fips.rs` | FIPS-validated cryptography |
| `zk.rs` | Zero-knowledge proof generation |
| `db.rs` | PostgreSQL connection and queries |

---

## The Website

- Lives in your repo root as `index.html`
- Uses the Ethereal HTML5 UP template
- Serves as the project landing page
- Shows: what the tool does, tech stack, how it works, quick start
- Host it on GitHub Pages for free:
  - Go to repo Settings → Pages → Source: main branch → root folder

---

## How to Run It

### Locally (development)
```powershell
# 1. Start database
docker compose up -d db

# 2. Run migrations
sqlx migrate run

# 3. Start the app
cargo run
```
App runs at: http://localhost:3000

### With Docker (full stack)
```powershell
docker compose up --build
```

### The Webhook Flow (when live)
1. You register your server URL in GitHub App settings:
   `https://your-server.com/webhook`
2. Developer opens a PR on any repo that installed your app
3. GitHub sends a POST request to your server
4. Your server verifies the HMAC signature
5. Runs the 3-layer analysis
6. Posts comments directly on the PR

---

## Environment Variables Required

| Variable | Where to get it |
|----------|----------------|
| `DATABASE_URL` | Your Postgres connection string |
| `OPENAI_API_KEY` | platform.openai.com |
| `GITHUB_APP_ID` | GitHub → Settings → Developer Settings → GitHub Apps |
| `PRIVATE_KEY_PATH` | Downloaded from your GitHub App settings |

---

