-- Migration: Add FHIR Search Helper Functions
-- Description: Create SQL functions to support FHIR search operations

-- Function to search patients by name
CREATE OR REPLACE FUNCTION fhir.search_patient_by_name(search_text TEXT)
RETURNS TABLE(id UUID, resource JSONB) AS $$
BEGIN
    RETURN QUERY
    SELECT p.id, p.resource
    FROM fhir.patient p
    WHERE (
        p.resource #>> '{name,0,family}' ILIKE '%' || search_text || '%'
        OR p.resource #>> '{name,0,given,0}' ILIKE '%' || search_text || '%'
        OR p.resource #>> '{name,0,text}' ILIKE '%' || search_text || '%'
    )
    AND p.status = 'created';
END;
$$ LANGUAGE plpgsql;

-- Function to search patients by gender
CREATE OR REPLACE FUNCTION fhir.search_patient_by_gender(gender_code TEXT)
RETURNS TABLE(id UUID, resource JSONB) AS $$
BEGIN
    RETURN QUERY
    SELECT p.id, p.resource
    FROM fhir.patient p
    WHERE p.resource->>'gender' = gender_code
    AND p.status = 'created';
END;
$$ LANGUAGE plpgsql;

-- Function to search patients by birth date
CREATE OR REPLACE FUNCTION fhir.search_patient_by_birthdate(birth_date DATE)
RETURNS TABLE(id UUID, resource JSONB) AS $$
BEGIN
    RETURN QUERY
    SELECT p.id, p.resource
    FROM fhir.patient p
    WHERE p.resource->>'birthDate' = birth_date::TEXT
    AND p.status = 'created';
END;
$$ LANGUAGE plpgsql;

-- Function to search patients by identifier
CREATE OR REPLACE FUNCTION fhir.search_patient_by_identifier(identifier_value TEXT)
RETURNS TABLE(id UUID, resource JSONB) AS $$
BEGIN
    RETURN QUERY
    SELECT p.id, p.resource
    FROM fhir.patient p
    WHERE p.resource @> jsonb_build_array(
        jsonb_build_object('identifier', jsonb_build_array(
            jsonb_build_object('value', identifier_value)
        ))
    )
    AND p.status = 'created';
END;
$$ LANGUAGE plpgsql;

-- Function to get patient by ID
CREATE OR REPLACE FUNCTION fhir.get_patient(patient_id UUID)
RETURNS JSONB AS $$
BEGIN
    RETURN (
        SELECT p.resource
        FROM fhir.patient p
        WHERE p.id = patient_id
        AND p.status = 'created'
        LIMIT 1
    );
END;
$$ LANGUAGE plpgsql;
