-- Migration: Initial FHIR Patient Schema
-- Description: Create main patient table with JSONB storage and supporting indexes

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create FHIR schema if it doesn't exist
CREATE SCHEMA IF NOT EXISTS fhir;

-- Main table for storing patients
CREATE TABLE IF NOT EXISTS fhir.patient (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_type VARCHAR(50) NOT NULL DEFAULT 'Patient',
    resource JSONB NOT NULL,
    txid BIGINT NOT NULL DEFAULT txid_current(),
    ts TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    status VARCHAR(20) DEFAULT 'created',
    CONSTRAINT check_resource_type CHECK (resource->>'resourceType' = 'Patient')
);

-- Change history table
CREATE TABLE IF NOT EXISTS fhir.patient_history (
    id UUID NOT NULL,
    version_id INTEGER NOT NULL,
    resource JSONB NOT NULL,
    txid BIGINT NOT NULL,
    ts TIMESTAMP WITH TIME ZONE NOT NULL,
    status VARCHAR(20),
    PRIMARY KEY (id, version_id)
);

-- GIN indexes for fast JSONB search
CREATE INDEX IF NOT EXISTS idx_patient_resource_gin 
    ON fhir.patient USING gin(resource jsonb_path_ops);

CREATE INDEX IF NOT EXISTS idx_patient_identifier 
    ON fhir.patient USING gin((resource->'identifier') jsonb_path_ops);

CREATE INDEX IF NOT EXISTS idx_patient_name 
    ON fhir.patient USING gin((resource->'name') jsonb_path_ops);

-- Indexes for frequent search queries
CREATE INDEX IF NOT EXISTS idx_patient_family 
    ON fhir.patient ((resource#>>'{name,0,family}'));

CREATE INDEX IF NOT EXISTS idx_patient_given 
    ON fhir.patient USING gin((resource#>'{name,0,given}'));

CREATE INDEX IF NOT EXISTS idx_patient_birthdate 
    ON fhir.patient ((resource->>'birthDate'));

CREATE INDEX IF NOT EXISTS idx_patient_gender 
    ON fhir.patient ((resource->>'gender'));

CREATE INDEX IF NOT EXISTS idx_patient_active 
    ON fhir.patient ((resource->>'active'));

-- Index for searching by identifiers
CREATE INDEX IF NOT EXISTS idx_patient_identifier_value 
    ON fhir.patient USING btree((resource#>>'{identifier,0,value}'));

-- Timestamp index
CREATE INDEX IF NOT EXISTS idx_patient_ts 
    ON fhir.patient (ts DESC);

-- Function to update timestamp on changes
CREATE OR REPLACE FUNCTION fhir.update_patient_ts()
RETURNS TRIGGER AS $$
BEGIN
    NEW.ts = NOW();
    NEW.txid = txid_current();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update timestamp
DROP TRIGGER IF EXISTS patient_ts_trigger ON fhir.patient;
CREATE TRIGGER patient_ts_trigger
BEFORE UPDATE ON fhir.patient
FOR EACH ROW
EXECUTE FUNCTION fhir.update_patient_ts();

-- Function to log changes to history
CREATE OR REPLACE FUNCTION fhir.log_patient_change()
RETURNS TRIGGER AS $$
DECLARE
    v_version_id INTEGER;
BEGIN
    SELECT COALESCE(MAX(version_id), 0) + 1 INTO v_version_id
    FROM fhir.patient_history
    WHERE id = NEW.id;
    
    INSERT INTO fhir.patient_history (id, version_id, resource, txid, ts, status)
    VALUES (NEW.id, v_version_id, NEW.resource, NEW.txid, NEW.ts, NEW.status);
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to log all changes
DROP TRIGGER IF EXISTS patient_history_trigger ON fhir.patient;
CREATE TRIGGER patient_history_trigger
AFTER INSERT OR UPDATE ON fhir.patient
FOR EACH ROW
EXECUTE FUNCTION fhir.log_patient_change();
