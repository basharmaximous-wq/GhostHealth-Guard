

```markdown id="blockchain-md"
# Blockchain Notarization

GhostHealth Guard can notarize **audit logs on a public blockchain** (Ethereum).

---

## Workflow

1. Compute **Merkle root** from audit log hashes  
2. Submit transaction to Ethereum smart contract:
   - Store Merkle root  
   - Optional metadata (tenant, timestamp)  

3. Store **transaction hash** in PostgreSQL for reference  

```text
previous_hash -> current_hash -> merkle_root -> blockchain_tx
```
Example
```
let merkle_root = blockchain::compute_merkle_root(audit_records);
let tx_hash = blockchain::submit_merkle_root(merkle_root);
sqlx::query!(
    "UPDATE audit_logs SET blockchain_tx = $1 WHERE id = $2",
    tx_hash,
    audit_id
)
.execute(&db)
.await?;``
