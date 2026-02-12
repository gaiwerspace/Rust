-- Run all migrations in sequence
-- Execute from project root: psql -d fhir_db -f migrations/run_migrations.sql

\echo 'Running migration 000_fhir_resources_table.sql...'
\i migrations/000_fhir_resources_table.sql

\echo 'Running migration 001_initial_schema.sql...'
\i migrations/001_initial_schema.sql

\echo 'Running migration 002_add_search_functions.sql...'
\i migrations/002_add_search_functions.sql

\echo 'Running migration 002_fhir_extension_functions.sql...'
\i migrations/002_fhir_extension_functions.sql

\echo 'Running migration 003_fhir_search_helpers.sql...'
\i migrations/003_fhir_search_helpers.sql

\echo 'All migrations completed successfully!'
