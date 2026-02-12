-- Initialize the FHIR database

-- Create the UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create the main table for FHIR resources
CREATE TABLE IF NOT EXISTS fhir_resources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_type VARCHAR(50) NOT NULL,
    resource_data JSONB NOT NULL,
    version_id INTEGER DEFAULT 1,
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for better search performance
CREATE INDEX IF NOT EXISTS idx_fhir_resources_type ON fhir_resources(resource_type);
CREATE INDEX IF NOT EXISTS idx_fhir_resources_data ON fhir_resources USING GIN(resource_data);

-- Create indexes for common search patterns
CREATE INDEX IF NOT EXISTS idx_patient_gender ON fhir_resources((resource_data->>'gender')) WHERE resource_type = 'Patient';
CREATE INDEX IF NOT EXISTS idx_patient_birthdate ON fhir_resources((resource_data->>'birthDate')) WHERE resource_type = 'Patient';
CREATE INDEX IF NOT EXISTS idx_patient_name ON fhir_resources USING GIN((resource_data->'name')) WHERE resource_type = 'Patient';

-- Create a view for easier querying
CREATE OR REPLACE VIEW patient_view AS
SELECT 
    id,
    resource_data,
    resource_data->>'gender' as gender,
    resource_data->>'birthDate' as birth_date,
    resource_data->'name'->0->>'family' as family_name,
    resource_data->'name'->0->'given'->0 as given_name,
    last_updated
FROM fhir_resources 
WHERE resource_type = 'Patient';

-- Load the FHIR extension (pgrx)
-- Note: The extension must be installed first using: cargo pgrx install
CREATE EXTENSION IF NOT EXISTS fhir_extension;