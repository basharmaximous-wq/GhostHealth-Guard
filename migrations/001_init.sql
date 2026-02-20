CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_name TEXT NOT NULL,
    pr_number INT NOT NULL,
    status TEXT NOT NULL,
    risk_score INT NOT NULL,
    report JSONB NOT NULL,
    previous_hash TEXT,
    current_hash TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

ALTER TABLE audit_logs ADD COLUMN blockchain_tx TEXT;
