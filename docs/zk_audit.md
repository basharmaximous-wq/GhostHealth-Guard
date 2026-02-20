# Audit Chain & ZK Proof

GhostHealth Guard generates a **tamper-proof audit chain** using **ZK proofs**.

---

## Workflow

1. **Pull Request Trigger** – Developer opens a PR  
2. **Static Analysis** – Semgrep rules detect PHI leaks  
3. **Architectural PHI Tracking** – Vector DB checks struct usage across repo  
4. **Audit Log Record** – Compute:
   ```text
   previous_hash + current_record -> SHA256 -> current_hash
