
-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Tenants table for multi-tenant SaaS
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    api_key TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Audit logs table
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(id),
    repo_name TEXT NOT NULL,
    pr_number INT NOT NULL,
    status TEXT NOT NULL, -- CLEAN / VIOLATION
    risk_score INT NOT NULL,
    report JSONB NOT NULL,
    previous_hash TEXT,
    current_hash TEXT NOT NULL,
    blockchain_tx TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
