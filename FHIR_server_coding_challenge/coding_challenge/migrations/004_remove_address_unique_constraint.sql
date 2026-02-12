-- Remove the overly restrictive UNIQUE constraint on patient_address_index
-- This allows patients to have multiple addresses in the same city/country

-- Drop the unique constraint
ALTER TABLE fhir.patient_address_index 
DROP CONSTRAINT IF EXISTS patient_address_index_id_city_country_key;

-- The table now allows multiple addresses with the same city/country for a patient
-- which is the correct behavior per FHIR specification
