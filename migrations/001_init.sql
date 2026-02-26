-- 1. Enable UUID extension first
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- 2. Create Tenants table
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    api_key TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 3. Create Audit logs table
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(id),
    repo_name TEXT NOT NULL,
    pr_number INT NOT NULL,
    status TEXT NOT NULL, 
    risk_score INT NOT NULL,
    report JSONB NOT NULL,
    previous_hash TEXT, 
    current_hash TEXT NOT NULL, 
    blockchain_tx TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 4. Create Index
CREATE INDEX IF NOT EXISTS idx_audit_hashes ON audit_logs(current_hash);

-- 5. NOW insert the default tenant
INSERT INTO tenants (name, api_key) 
VALUES ('My Local Lab', 'dev-key-12345') 
ON CONFLICT (api_key) DO NOTHING;