-- Migration: FHIR Search Helper Tables and Indexes
-- Description: Add denormalized search indexes and helper tables for optimal query performance

-- Search index table for patient names (denormalized for fast searching)
CREATE TABLE IF NOT EXISTS fhir.patient_name_index (
    idx_id BIGSERIAL PRIMARY KEY,
    id UUID NOT NULL,
    family_name VARCHAR(255),
    given_name VARCHAR(255),
    name_use VARCHAR(50),
    UNIQUE (id, family_name, given_name),
    FOREIGN KEY (id) REFERENCES fhir.patient(id) ON DELETE CASCADE
);

-- Search index table for patient identifiers
CREATE TABLE IF NOT EXISTS fhir.patient_identifier_index (
    idx_id BIGSERIAL PRIMARY KEY,
    id UUID NOT NULL,
    identifier_value VARCHAR(255),
    identifier_type VARCHAR(100),
    identifier_system VARCHAR(255),
    UNIQUE (id, identifier_value),
    FOREIGN KEY (id) REFERENCES fhir.patient(id) ON DELETE CASCADE
);

-- Search index table for patient telecom (contact points)
CREATE TABLE IF NOT EXISTS fhir.patient_telecom_index (
    idx_id BIGSERIAL PRIMARY KEY,
    id UUID NOT NULL,
    telecom_value VARCHAR(255),
    telecom_system VARCHAR(50),
    telecom_use VARCHAR(50),
    UNIQUE (id, telecom_value),
    FOREIGN KEY (id) REFERENCES fhir.patient(id) ON DELETE CASCADE
);

-- Search index table for patient addresses
CREATE TABLE IF NOT EXISTS fhir.patient_address_index (
    idx_id BIGSERIAL PRIMARY KEY,
    id UUID NOT NULL,
    city VARCHAR(255),
    country VARCHAR(100),
    postal_code VARCHAR(20),
    address_text TEXT,
    FOREIGN KEY (id) REFERENCES fhir.patient(id) ON DELETE CASCADE
);

-- Create indexes for fast searching on denormalized tables
CREATE INDEX IF NOT EXISTS idx_patient_name_family_idx 
    ON fhir.patient_name_index(family_name);

CREATE INDEX IF NOT EXISTS idx_patient_name_given_idx 
    ON fhir.patient_name_index(given_name);

CREATE INDEX IF NOT EXISTS idx_patient_identifier_value_idx 
    ON fhir.patient_identifier_index(identifier_value);

CREATE INDEX IF NOT EXISTS idx_patient_identifier_system_idx 
    ON fhir.patient_identifier_index(identifier_system);

CREATE INDEX IF NOT EXISTS idx_patient_telecom_value_idx 
    ON fhir.patient_telecom_index(telecom_value);

CREATE INDEX IF NOT EXISTS idx_patient_address_city_idx 
    ON fhir.patient_address_index(city);

CREATE INDEX IF NOT EXISTS idx_patient_address_country_idx 
    ON fhir.patient_address_index(country);

-- Full text search indexes on JSONB for advanced search
CREATE INDEX IF NOT EXISTS idx_patient_name_tsvector 
    ON fhir.patient USING GIN(
        to_tsvector('english', 
            COALESCE(resource#>>'{name,0,family}', '') || ' ' ||
            COALESCE(resource#>>'{name,0,given,0}', '')
        )
    );

-- Function to maintain patient name index
CREATE OR REPLACE FUNCTION fhir.maintain_patient_name_index()
RETURNS TRIGGER AS $$
BEGIN
    -- Delete old entries
    DELETE FROM fhir.patient_name_index WHERE id = NEW.id;
    
    -- Insert new entries from JSONB array - only if at least family or given is provided
    INSERT INTO fhir.patient_name_index (id, family_name, given_name, name_use)
    SELECT
        NEW.id,
        name_elem->>'family',
        (name_elem->'given'->>0),
        name_elem->>'use'
    FROM jsonb_array_elements(NEW.resource->'name') AS name_elem
    WHERE (name_elem->>'family') IS NOT NULL OR (name_elem->'given'->>0) IS NOT NULL;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Function to maintain patient identifier index
CREATE OR REPLACE FUNCTION fhir.maintain_patient_identifier_index()
RETURNS TRIGGER AS $$
BEGIN
    -- Delete old entries
    DELETE FROM fhir.patient_identifier_index WHERE id = NEW.id;
    
    -- Insert new entries from JSONB array - only if value is provided
    INSERT INTO fhir.patient_identifier_index (id, identifier_value, identifier_type, identifier_system)
    SELECT
        NEW.id,
        id_elem->>'value',
        id_elem#>>'{type,coding,0,code}',
        id_elem->>'system'
    FROM jsonb_array_elements(NEW.resource->'identifier') AS id_elem
    WHERE (id_elem->>'value') IS NOT NULL;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Function to maintain patient telecom index
CREATE OR REPLACE FUNCTION fhir.maintain_patient_telecom_index()
RETURNS TRIGGER AS $$
BEGIN
    -- Delete old entries
    DELETE FROM fhir.patient_telecom_index WHERE id = NEW.id;
    
    -- Insert new entries from JSONB array - only if value is provided
    INSERT INTO fhir.patient_telecom_index (id, telecom_value, telecom_system, telecom_use)
    SELECT
        NEW.id,
        tel_elem->>'value',
        tel_elem->>'system',
        tel_elem->>'use'
    FROM jsonb_array_elements(NEW.resource->'telecom') AS tel_elem
    WHERE (tel_elem->>'value') IS NOT NULL;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Function to maintain patient address index
CREATE OR REPLACE FUNCTION fhir.maintain_patient_address_index()
RETURNS TRIGGER AS $$
BEGIN
    -- Delete old entries
    DELETE FROM fhir.patient_address_index WHERE id = NEW.id;
    
    -- Insert new entries from JSONB array - only if at least city or country is provided
    INSERT INTO fhir.patient_address_index (id, city, country, postal_code, address_text)
    SELECT
        NEW.id,
        addr_elem->>'city',
        addr_elem->>'country',
        addr_elem->>'postalCode',
        addr_elem->>'text'
    FROM jsonb_array_elements(NEW.resource->'address') AS addr_elem
    WHERE (addr_elem->>'city' IS NOT NULL OR addr_elem->>'country' IS NOT NULL OR addr_elem->>'text' IS NOT NULL);
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create triggers to maintain search indexes
DROP TRIGGER IF EXISTS patient_name_index_trigger ON fhir.patient;
CREATE TRIGGER patient_name_index_trigger
AFTER INSERT OR UPDATE ON fhir.patient
FOR EACH ROW
EXECUTE FUNCTION fhir.maintain_patient_name_index();

DROP TRIGGER IF EXISTS patient_identifier_index_trigger ON fhir.patient;
CREATE TRIGGER patient_identifier_index_trigger
AFTER INSERT OR UPDATE ON fhir.patient
FOR EACH ROW
EXECUTE FUNCTION fhir.maintain_patient_identifier_index();

DROP TRIGGER IF EXISTS patient_telecom_index_trigger ON fhir.patient;
CREATE TRIGGER patient_telecom_index_trigger
AFTER INSERT OR UPDATE ON fhir.patient
FOR EACH ROW
EXECUTE FUNCTION fhir.maintain_patient_telecom_index();

DROP TRIGGER IF EXISTS patient_address_index_trigger ON fhir.patient;
CREATE TRIGGER patient_address_index_trigger
AFTER INSERT OR UPDATE ON fhir.patient
FOR EACH ROW
EXECUTE FUNCTION fhir.maintain_patient_address_index();
