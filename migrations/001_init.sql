CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    repo_name TEXT NOT NULL,
    pr_number INT NOT NULL,
    status TEXT NOT NULL,
    risk_score INT NOT NULL,
    report JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);