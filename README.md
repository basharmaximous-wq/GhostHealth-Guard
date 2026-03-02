# 🛡️ GhostHealth Guard
![GhostHealth Logo](https://raw.githubusercontent.com/basharmaximous-wq/GhostHealth-Guard/master/docs/assets/logo.png)
### **Zero‑stress HIPAA compliance for Rust health‑tech teams.**

---

## 🚀 Catch HIPAA compliance issues before they cost you $50,000 — inside your GitHub PRs.

GhostHealth Guard is the **invisible shield** for your Rust health-tech stack. It’s an AI-powered sentinel that hunts for PHI leaks, enforces HIPAA-ready architecture, and silences privacy risks before they ever reach production.

---

## 📈 Why this matters: The Cost of a Single Mistake
In high-compliance industries like Health Informatics, a simple `println!` of a `PatientName` or `SocialSecurityNumber` can lead to **massive legal liabilities** and trust erosion. 

- **The Risk:** Standard linters don't understand the difference between a "Username" (public) and "PatientID" (PHI).
- **The Gap:** A single accidental log entry can trigger **HIPAA fines starting at $50,000** per violation.
- **The Solution:** A specialized, context-aware auditor that understands both Rust's memory safety and HIPAA/GDPR privacy requirements.

---

## ⚙️ How it works: Multi-Layered Security

1. **Static Privacy Scanning:** Uses custom Semgrep rules to catch hardcoded sensitive strings or unmasked logging of PHI.
2. **AI-Powered Code Review:** A specialized LLM (Gemini 1.5 Flash/GPT-4o) reviews the logic for "Privacy Smells," such as unencrypted data transit or improper implementation of the `Debug` trait on sensitive data.
3. **Architectural PHI Tracking:** Uses a Vector Database (RAG) to understand your codebase's data models. It knows if a struct is `#[Sensitive]` and flags any unsafe handling across the whole repo.
4. **Confidential Computing:** Optional **SGX enclave** support for processing audit logs in a secure, hardware-isolated environment.
5. **Immutable Audit Chain:** Every scan result is notarized on a blockchain and linked via **ZK-proofs**, ensuring your compliance history is tamper-proof for auditors.

### **The Workflow**
```text
Developer -> GitHub PR -> GhostHealth Scan -> Flags Issues -> Secure Merge ✅
```

---

## 🛠️ Installation

### 1. Clone & Setup
```bash
git clone https://github.com/basharmaximous-wq/GhostHealth-Guard.git
cd GhostHealth-Guard
cargo install --path .
```

### 2. Configure Environment Variables
Create a `.env` file with the following:
```bash
DATABASE_URL=postgres://user:pass@localhost:5432/ghostdb
GEMINI_API_KEY=your_gemini_api_key_here
GITHUB_APP_ID=your_app_id
PRIVATE_KEY_PATH=path/to/key.pem
```

### 3. Initialize Database
```bash
sqlx migrate run
```

---

## 💻 Using the App Locally

You can run a standalone audit on any Rust file or directory:

```bash
ghosthealth-guard scan ./src --api-key $GEMINI_API_KEY
```

**Output Example:**
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

## 🔗 GitHub Integration Example
Add this snippet to `.github/workflows/ghosthealth_check.yml` to automate compliance on every PR:

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

![Pull Request Comment Screenshot](https://raw.githubusercontent.com/basharmaximous-wq/GhostHealth-Guard/master/docs/assets/scan_result.png)

---

## 💸 Why this saves money: The ROI of Privacy
GhostHealth Guard isn't just a tool; it's an **insurance policy**.

| Scenario | Cost without Guard | Cost with GhostHealth Guard |
|----------|-------------------|-----------------------------|
| PHI Leak in Production | **$50,000+ Fine** | **$0** (Caught in PR) |
| Manual Security Audit | **$10,000 / week** | **Included** (Auto-reports) |
| Developer Time | **~5h / week** (Fixing leaks late) | **Minutes** (Fixing at source) |

**ROI:** A single caught violation pays for years of development and AI API costs.

---

## 🧠 Who it’s for:
- 🦀 **Rust Health‑Tech Developers:** Building EHR systems, medical devices, or PHI-aware apps.
- 🚀 **Startups:** Small teams that need to meet enterprise-grade compliance without a dedicated security staff.
- 🌍 **Open‑Source Projects:** Protecting the integrity of decentralized or public health data.

---

## ❓ Troubleshooting & FAQ
- **Missing API key?** Ensure `GEMINI_API_KEY` or `OPENAI_API_KEY` is exported in your environment.
- **Timeout on large repos?** Use the `--timeout` flag to increase processing time for deep architectural analysis.
- **False positive?** Suppress specific warnings using the `#[allow(compliance_risk)]` or `#[ignore_compliance]` attributes on your functions.

---

## 🤝 Contributing & License
We welcome contributors! Check out [CONTRIBUTING.md](CONTRIBUTING.md) to get started.
Licensed under **Apache-2.0**.

---
*GhostHealth Guard: Don't just ship code; ship compliance.*
