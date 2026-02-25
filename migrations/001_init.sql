-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Tenants table
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    api_key TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Audit logs table (The "Chain")
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(id),
    repo_name TEXT NOT NULL,
    pr_number INT NOT NULL,
    status TEXT NOT NULL, -- 'CLEAN' or 'VIOLATION'
    risk_score INT NOT NULL,
    report JSONB NOT NULL,
    previous_hash TEXT, -- The hash of the record created right before this one
    current_hash TEXT NOT NULL, -- The hash of THIS record
    blockchain_tx TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for performance
CREATE INDEX idx_audit_hashes ON audit_logs(current_hash);