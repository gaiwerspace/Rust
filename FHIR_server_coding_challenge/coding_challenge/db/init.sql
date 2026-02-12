-- Database initialization script
-- This script runs migrations to set up the database schema

-- Run migrations
\echo 'Initializing database schema...'
\i /build/migrations/001_initial_schema.sql
\i /build/migrations/002_add_search_functions.sql

\echo 'Database initialization complete!'

-- Create extension if it exists
CREATE EXTENSION IF NOT EXISTS uuid-ossp;

-- Main FHIR resources table with JSONB storage
CREATE TABLE IF NOT EXISTS fhir_resources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_type VARCHAR(50) NOT NULL,
    resource_data JSONB NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_resource_type ON fhir_resources(resource_type);
CREATE INDEX IF NOT EXISTS idx_created_at ON fhir_resources(created_at);
CREATE INDEX IF NOT EXISTS idx_updated_at ON fhir_resources(updated_at);

-- JSONB indexes for common search fields
CREATE INDEX IF NOT EXISTS idx_patient_name 
    ON fhir_resources USING GIN(resource_data->'name') 
    WHERE resource_type = 'Patient';

CREATE INDEX IF NOT EXISTS idx_patient_gender 
    ON fhir_resources USING GIN(resource_data->'gender') 
    WHERE resource_type = 'Patient';

CREATE INDEX IF NOT EXISTS idx_patient_birthdate 
    ON fhir_resources USING GIN(resource_data->'birthDate') 
    WHERE resource_type = 'Patient';

CREATE INDEX IF NOT EXISTS idx_patient_identifier 
    ON fhir_resources USING GIN(resource_data->'identifier') 
    WHERE resource_type = 'Patient';

CREATE INDEX IF NOT EXISTS idx_patient_active 
    ON fhir_resources USING GIN(resource_data->'active') 
    WHERE resource_type = 'Patient';

-- Full JSONB index for flexible queries
CREATE INDEX IF NOT EXISTS idx_fhir_resources_jsonb 
    ON fhir_resources USING GIN(resource_data);

-- Function to update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_fhir_resources_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to automatically update updated_at
DROP TRIGGER IF EXISTS fhir_resources_updated_at_trigger ON fhir_resources;
CREATE TRIGGER fhir_resources_updated_at_trigger
BEFORE UPDATE ON fhir_resources
FOR EACH ROW
EXECUTE FUNCTION update_fhir_resources_updated_at();

-- Fallback SQL functions (for when extension is not available)
CREATE OR REPLACE FUNCTION fhir_put(
    p_resource_type VARCHAR,
    p_resource_data JSONB
) RETURNS UUID AS $$
DECLARE
    v_id UUID;
BEGIN
    INSERT INTO fhir_resources (resource_type, resource_data)
    VALUES (p_resource_type, p_resource_data)
    RETURNING id INTO v_id;
    RETURN v_id;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION fhir_get(
    p_resource_type VARCHAR,
    p_resource_id UUID
) RETURNS JSONB AS $$
BEGIN
    RETURN (
        SELECT resource_data
        FROM fhir_resources
        WHERE resource_type = p_resource_type
        AND id = p_resource_id
        LIMIT 1
    );
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION fhir_search(
    p_resource_type VARCHAR,
    p_param VARCHAR,
    p_op VARCHAR,
    p_value VARCHAR
) RETURNS TABLE(id UUID) AS $$
BEGIN
    RETURN QUERY
    SELECT fr.id
    FROM fhir_resources fr
    WHERE fr.resource_type = p_resource_type
    AND (
        (p_param = 'name' AND fr.resource_data->'name' @> to_jsonb(array[jsonb_build_object('given', array[p_value::text])]))
        OR (p_param = 'gender' AND fr.resource_data->>'gender' = p_value)
        OR (p_param = 'birthDate' AND fr.resource_data->>'birthDate' = p_value)
        OR (p_param = '_id' AND fr.id::text = p_value)
    );
END;
$$ LANGUAGE plpgsql;
