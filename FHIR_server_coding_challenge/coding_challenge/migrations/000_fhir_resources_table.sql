-- Table expected by the Rust server (database.rs)
-- Creates fhir_resources in public schema; migrations/001+ use fhir.patient.

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS fhir_resources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_type VARCHAR(50) NOT NULL,
    resource_data JSONB NOT NULL,
    version_id INTEGER DEFAULT 1,
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_fhir_resources_type ON fhir_resources(resource_type);
CREATE INDEX IF NOT EXISTS idx_fhir_resources_data ON fhir_resources USING GIN(resource_data);
CREATE INDEX IF NOT EXISTS idx_patient_gender ON fhir_resources((resource_data->>'gender')) WHERE resource_type = 'Patient';
CREATE INDEX IF NOT EXISTS idx_patient_birthdate ON fhir_resources((resource_data->>'birthDate')) WHERE resource_type = 'Patient';
CREATE INDEX IF NOT EXISTS idx_patient_name ON fhir_resources USING GIN((resource_data->'name')) WHERE resource_type = 'Patient';
