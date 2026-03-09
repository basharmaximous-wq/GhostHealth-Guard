<p align="center">
  <img src="https://readme-typing-svg.demolab.com?font=Orbitron&weight=900&size=36&pause=1000&color=00FFE7&center=true&vCenter=true&width=700&height=80&lines=GhostHealth+Guard+%F0%9F%9B%A1%EF%B8%8F;Zero-Stress+HIPAA+Compliance;Ship+Code.+Ship+Compliance." alt="GhostHealth Guard" />
</p>

<p align="center">
  <strong>The invisible shield for your Rust health-tech stack.</strong><br/>
  AI-powered HIPAA sentinel that hunts PHI leaks, enforces privacy architecture,<br/>and silences compliance risks before they ever reach production.
</p>

<br/>

<p align="center">
  <a href="https://github.com/basharmaximous-wq/GhostHealth-Guard/releases/latest"><img src="https://img.shields.io/github/v/release/basharmaximous-wq/GhostHealth-Guard?style=for-the-badge&logo=github&color=00ffe7&labelColor=020812&label=Latest+Release" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-39ff14?style=for-the-badge&labelColor=020812" /></a>
  <a href="#"><img src="https://img.shields.io/badge/Language-Rust_🦀-ff2d78?style=for-the-badge&labelColor=020812" /></a>
  <a href="#"><img src="https://img.shields.io/badge/HIPAA-Compliant_✅-00ffe7?style=for-the-badge&labelColor=020812" /></a>
  <a href="#"><img src="https://img.shields.io/badge/AI_Engine-Gemini_%7C_GPT--4o-bf5fff?style=for-the-badge&labelColor=020812" /></a>
</p>

<p align="center">
  <a href="https://basharmaximous-wq.github.io/GhostHealth-Guard/">
    <img src="https://img.shields.io/badge/🌐_Live_Demo-View_Animated_Page-00ffe7?style=for-the-badge&labelColor=020812" />
  </a>
</p>

---

## 🚀 Catch HIPAA violations before they cost you $50,000 — inside your GitHub PRs

> A single accidental `println!` of a `SocialSecurityNumber` can trigger **HIPAA fines starting at $50,000 per violation.**
> Standard linters don't know the difference between `"Username"` (public) and `"PatientID"` (PHI). GhostHealth Guard does.

---

## ⚙️ How It Works — Multi-Layered Security

| # | Layer | What It Does |
|---|-------|--------------|
| 1️⃣ | **Static Privacy Scanning** | Custom Semgrep rules catch hardcoded PHI and unmasked logging |
| 2️⃣ | **AI-Powered Code Review** | Gemini 1.5 Flash / GPT-4o hunts "Privacy Smells" — unencrypted transit, bad `Debug` trait usage |
| 3️⃣ | **Architectural PHI Tracking** | Vector DB (RAG) tracks `#[Sensitive]` structs across your entire repo |
| 4️⃣ | **Confidential Computing** | Optional Intel SGX enclave — even Root can't read your audit logs |
| 5️⃣ | **Immutable Audit Chain** | Every scan notarized on blockchain + ZK-proofs. Tamper-proof forever |

```
Developer ──▶ GitHub PR ──▶ GhostHealth Scan ──▶ Flags Issues ──▶ Secure Merge ✅
                                    │
                    ┌───────────────┼───────────────────┐
                    ▼               ▼                   ▼
             Static Scan       AI Review         Audit Chain
           (Semgrep rules)  (Gemini/GPT-4o)   (ZK-proof hash)
```

---

## 🛡️ Core Security Features

- 🔮 **Zero-Knowledge (ZK) Proofs** — Prove compliance to auditors without exposing your code logic
- 🏛️ **FIPS 140-2/3 Readiness** — Cryptographic modules meeting federal security standards
- ⚙️ **Hardware Enclaves (TEE / Intel SGX)** — Audit logs processed in a hardware-isolated black box
- ⛓️ **Immutable Blockchain-Notarized Logs** — No retroactive tampering, ever

---

## 📋 HIPAA Production Readiness

| HIPAA Safeguard | Regulation | GhostHealth Guard Feature |
|---|---|---|
| Access Control | §164.312(a)(1) | Automated SGX Enclave isolation |
| Audit Controls | §164.312(b) | Immutable blockchain-notarized audit logs |
| Integrity | §164.312(c)(1) | ZK-Proofs on every scan result |
| Transmission Security | §164.312(e)(1) | Static analysis for TLS/SSL errors |

---

## 📦 Multi-Platform Support

| Platform | Architectures |
|---|---|
| 🐧 **Linux** | x86_64, AArch64 (Ubuntu / Debian / CentOS) |
| 🍎 **macOS** | Intel & Apple Silicon (M1 / M2 / M3) |


**[⬇️ Download Latest Binary →](https://github.com/basharmaximous-wq/GhostHealth-Guard/releases/latest)**

---

## 🛠️ Installation

### ⚡ Quick Start — No Rust Required

1. Download the binary for your OS from [Latest Release](https://github.com/basharmaximous-wq/GhostHealth-Guard/releases/latest)
2. Extract the file
3. Move to your path:
   ```bash
   mv ghosthealth-guard /usr/local/bin   # Linux/macOS
   # or add to PATH on Windows
   ```
4. Verify:
   ```bash
   ghosthealth-guard --version
   ```

### 🦀 Build from Source

```bash
git clone https://github.com/basharmaximous-wq/GhostHealth-Guard.git
cd GhostHealth-Guard
cargo install --path .
```

### 🔧 Configure Environment Variables

Create a `.env` file:

```bash
DATABASE_URL=postgres://user:pass@localhost:5432/ghostdb
GEMINI_API_KEY=your_gemini_api_key_here
GITHUB_APP_ID=your_app_id
PRIVATE_KEY_PATH=path/to/key.pem
```

### 🗄️ Initialize Database

```bash
sqlx migrate run
```

---

## 💻 Usage

### Run a Local Scan

```bash
ghosthealth-guard scan ./src --api-key $GEMINI_API_KEY
```

### Example Output

```json
{
  "status": "VIOLATION",
  "risk_score": 85,
  "issues": [
    {
      "category": "PHI_LOGGING",
      "severity": "HIGH",
      "message": "PHI field 'ssn' logged at line 42 — HIPAA violation"
    }
  ]
}
```

---

## 🔗 GitHub Actions — Automate on Every PR

Add this to `.github/workflows/ghosthealth_check.yml`:

```yaml
name: Compliance Audit
on: [pull_request]

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run GhostHealth Guard
        run: ghosthealth-guard scan-pr --pr-number ${{ github.event.number }}
        env:
          GEMINI_API_KEY: ${{ secrets.GEMINI_API_KEY }}
```

---

## 💸 The ROI of Privacy

| Scenario | ❌ Without Guard | ✅ With GhostHealth Guard |
|---|---|---|
| PHI Leak in Production | **$50,000+ Fine** | **$0** — Caught in PR |
| Manual Security Audit | **$10,000 / week** | **Included** — Auto-reports |
| Developer Time | **~5h / week** fixing leaks late | **Minutes** — fixing at source |

> 💡 A single caught violation pays for **years** of development and AI API costs.

---

## 🧠 Who It's For

- 🦀 **Rust Health-Tech Developers** — Building EHR systems, medical devices, or PHI-aware apps
- 🚀 **Startups** — Small teams needing enterprise-grade compliance without dedicated security staff
- 🌍 **Open-Source Projects** — Protecting decentralized or public health data integrity

---

## ❓ Troubleshooting & FAQ

<details>
<summary><strong>Missing API Key?</strong></summary>

Ensure `GEMINI_API_KEY` or `OPENAI_API_KEY` is exported in your environment before running any scan.
</details>

<details>
<summary><strong>Timeout on Large Repos?</strong></summary>

Use the `--timeout` flag to increase processing time for deep architectural analysis.
</details>

<details>
<summary><strong>Getting a False Positive?</strong></summary>

Suppress specific warnings with:
```rust
#[allow(compliance_risk)]
// or
#[ignore_compliance]
fn your_function() { ... }
```
</details>

<details>
<summary><strong>Database Connection Refused?</strong></summary>

GhostHealth Guard requires a live PostgreSQL instance. Verify your `DATABASE_URL` is correct and run:
```bash
sqlx migrate run
```
</details>

---

## 🤝 Contributing

We welcome contributors! Check out [CONTRIBUTING.md](CONTRIBUTING.md) to get started.

---

## 📄 License

Licensed under **[Apache-2.0](LICENSE)**.

---

<p align="center">
  <em>GhostHealth Guard — Don't just ship code; ship compliance.</em><br/><br/>
  <a href="https://github.com/basharmaximous-wq/GhostHealth-Guard/releases/latest"><img src="https://img.shields.io/badge/⬇️_Download-Latest_Release-00ffe7?style=for-the-badge&labelColor=020812" /></a>
  &nbsp;
  <a href="https://basharmaximous-wq.github.io/GhostHealth-Guard/"><img src="https://img.shields.io/badge/🌐_Live_Demo-Animated_Page-bf5fff?style=for-the-badge&labelColor=020812" /></a>
</p>
