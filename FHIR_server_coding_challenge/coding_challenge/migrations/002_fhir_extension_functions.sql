-- Migration: FHIR Extension Functions
-- Description: Create SQL functions to persist and retrieve FHIR resources via extension

-- Main FHIR resource storage function
-- All resource persistence goes through this function
CREATE OR REPLACE FUNCTION fhir.fhir_put(
    p_resource_type VARCHAR,
    p_resource_data JSONB
) RETURNS UUID AS $$
DECLARE
    v_id UUID;
    v_resource JSONB;
BEGIN
    -- Extract or generate ID
    v_id := COALESCE(
        (p_resource_data->>'id')::UUID,
        gen_random_uuid()
    );
    
    -- Ensure resource has required fields
    v_resource := jsonb_set(
        jsonb_set(p_resource_data, '{id}', to_jsonb(v_id::TEXT)),
        '{resourceType}',
        to_jsonb(p_resource_type)
    );
    
    -- Upsert into patient table - ONLY way to persist
    INSERT INTO fhir.patient (id, resource_type, resource, status)
    VALUES (v_id, p_resource_type, v_resource, 'created')
    ON CONFLICT (id) DO UPDATE SET
        resource = v_resource,
        status = 'created',
        ts = NOW(),
        txid = txid_current();
    
    RETURN v_id;
END;
$$ LANGUAGE plpgsql;

-- Retrieve FHIR resource by type and ID
-- All resource queries go through this function
CREATE OR REPLACE FUNCTION fhir.fhir_get(
    p_resource_type VARCHAR,
    p_resource_id UUID
) RETURNS JSONB AS $$
BEGIN
    RETURN (
        SELECT resource
        FROM fhir.patient
        WHERE resource_type = p_resource_type
        AND id = p_resource_id
        AND status = 'created'
        LIMIT 1
    );
END;
$$ LANGUAGE plpgsql;

-- Search FHIR resources by parameter
-- All search queries go through this function
CREATE OR REPLACE FUNCTION fhir.fhir_search(
    p_resource_type VARCHAR,
    p_param VARCHAR,
    p_op VARCHAR,
    p_value VARCHAR
) RETURNS TABLE(id UUID) AS $$
BEGIN
    RETURN QUERY
    SELECT DISTINCT p.id
    FROM fhir.patient p
    WHERE p.resource_type = p_resource_type
    AND p.status = 'created'
    AND (
        (p_param = 'name' AND p_op = 'contains' AND (
            (p.resource #>> '{name,0,family}' IS NOT NULL AND p.resource #>> '{name,0,family}' ILIKE '%' || p_value || '%')
            OR (p.resource #>> '{name,0,given,0}' IS NOT NULL AND p.resource #>> '{name,0,given,0}' ILIKE '%' || p_value || '%')
            OR EXISTS (
                SELECT 1 FROM jsonb_array_elements(p.resource->'name') AS name_elem
                WHERE (name_elem->>'family' IS NOT NULL AND name_elem->>'family' ILIKE '%' || p_value || '%')
                OR (name_elem->>'text' IS NOT NULL AND name_elem->>'text' ILIKE '%' || p_value || '%')
                OR EXISTS (
                    SELECT 1 FROM jsonb_array_elements_text(name_elem->'given') AS given_elem
                    WHERE given_elem ILIKE '%' || p_value || '%'
                )
            )
        ))
        OR (p_param = 'gender' AND p_op = 'exact' AND p.resource->>'gender' IS NOT NULL AND p.resource->>'gender' = p_value)
        OR (p_param = 'birthDate' AND p_op = 'eq' AND p.resource->>'birthDate' IS NOT NULL AND p.resource->>'birthDate' = p_value)
        OR (p_param = 'birthDate' AND p_op = 'ge' AND p.resource->>'birthDate' IS NOT NULL AND p.resource->>'birthDate' >= p_value)
        OR (p_param = 'birthDate' AND p_op = 'le' AND p.resource->>'birthDate' IS NOT NULL AND p.resource->>'birthDate' <= p_value)
        OR (p_param = 'active' AND p_op = 'exact' AND p.resource->>'active' IS NOT NULL AND p.resource->>'active' = p_value)
    )
    ORDER BY p.id;
END;
$$ LANGUAGE plpgsql;

-- Get patient history including current version
CREATE OR REPLACE FUNCTION fhir.fhir_get_history(
    p_patient_id UUID
) RETURNS TABLE(
    version_id INT,
    resource JSONB,
    ts TIMESTAMP WITH TIME ZONE,
    method VARCHAR
) AS $$
BEGIN
    RETURN QUERY
    SELECT 0::INT, p.resource, p.ts, 'PUT'::VARCHAR
    FROM fhir.patient p
    WHERE p.id = p_patient_id
    AND p.status = 'created'
    UNION ALL
    SELECT ph.version_id, ph.resource, ph.ts, 'PUT'::VARCHAR
    FROM fhir.patient_history ph
    WHERE ph.id = p_patient_id
    ORDER BY version_id DESC;
END;
$$ LANGUAGE plpgsql;

-- Update FHIR resource (increments version)
-- Usage: SELECT fhir.fhir_update('Patient', '550e8400-e29b-41d4-a716-446655440000'::uuid, '{"resourceType": "Patient", ...}'::jsonb)
CREATE OR REPLACE FUNCTION fhir.fhir_update(
    p_resource_type VARCHAR,
    p_resource_id UUID,
    p_resource_data JSONB
) RETURNS UUID AS $$
DECLARE
    v_resource JSONB;
BEGIN
    -- Ensure resource has correct ID and type
    v_resource := jsonb_set(
        jsonb_set(p_resource_data, '{id}', to_jsonb(p_resource_id::TEXT)),
        '{resourceType}',
        to_jsonb(p_resource_type)
    );
    
    -- Update main table
    UPDATE fhir.patient 
    SET resource = v_resource, 
        ts = NOW(), 
        txid = txid_current(),
        status = 'created'
    WHERE id = p_resource_id;
    
    -- If no rows updated, patient doesn't exist
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Patient with ID % not found', p_resource_id;
    END IF;
    
    RETURN p_resource_id;
END;
$$ LANGUAGE plpgsql;
