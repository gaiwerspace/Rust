# Running the FHIR Patient Server

## Quick Start Options

Rebuild the server: 
cargo build --release
Apply the migration: psql -d fhir_db -f migrations/002_fhir_extension_functions.sql
Restart the server: cargo run --bin fhir-server

### Option 1: Automated Setup (Recommended)
```bash
./setup.sh
```
This script will:
- Check prerequisites (PostgreSQL, Rust)
- Create the fhir_db database
- Run all migrations automatically
- Build the Rust server
- Display connection instructions
# 1. Ensure PostgreSQL is running
psql -U postgres -c "SELECT version();"

# 2. Set database URL
export DATABASE_URL="postgresql://postgres:password@localhost:5432/fhir_db"

# 3. Create database
psql -U postgres -c "CREATE DATABASE fhir_db;" 2>/dev/null || true

# 4. Run migrations (one time only)
cd /home/master/Alex/Web_Projects/Rust/Rust_test_project/coding_challenge
psql -d fhir_db -f migrations/run_migrations.sql

# 5. Build the server
cargo build --release

# 6. Run the local server
cargo run --bin fhir-server


### Option 2: Docker Compose
```bash
docker-compose up --build
```
This will:
- Start PostgreSQL 16 container
- Create the database and run migrations
- Start the FHIR server on http://localhost:3000

### Option 3: Manual Setup
```bash
# 1. Create database
psql -U postgres -c "CREATE DATABASE fhir_db;"

# 2. Run migrations
export DATABASE_URL=postgresql://postgres:password@localhost:5432/fhir_db
psql -d fhir_db -f migrations/run_migrations.sql

# 3. Build and run
cargo build --release
cargo run --bin fhir-server
```

## Database Initialization

The database is initialized via SQL migrations:
- `migrations/001_initial_schema.sql` - Core patient table and indexes
- `migrations/002_add_search_functions.sql` - Search helper functions
- `migrations/002_fhir_extension_functions.sql` - FHIR extension functions
- `migrations/003_fhir_search_helpers.sql` - Additional search helpers
- `migrations/run_migrations.sql` - Runs all migrations in sequence

## Architecture

The server uses **pure SQL PL/pgSQL functions** defined in migrations for all database operations:

- `fhir_put()` - Creates or updates FHIR resources with versioning
- `fhir_get()` - Retrieves resources by UUID or logical_id
- `fhir_search()` - Searches resources by parameters
- `fhir_count()` - Returns count of active resources
- `fhir_delete()` - Soft deletes resources
- `fhir_history()` - Retrieves version history
- `index_patient()` - Populates search indexes

## Rust Code Structure

The server code is organized as:
- `server/src/main.rs` - Axum HTTP server setup
- `server/src/database.rs` - Database layer (calls SQL functions)
- `server/src/handlers.rs` - HTTP request handlers
- `server/src/models.rs` - FHIR data structures

## Quick Start with Docker (Recommended if you have it set up, as it handles all dependencies automatically.)

The easiest way to run the server is with Docker Compose. This will build a custom PostgreSQL 16 image with the pgrx FHIR extension:

```bash
docker-compose up --build -d
```

This will:
- Build PostgreSQL 16 with the pgrx FHIR extension installed
- Create the FHIR database with required tables and indexes
- Start the FHIR server on port 3000

**Note:** The first build will take several minutes as it compiles Rust, pgrx, and the extension.

## Architecture

The server now uses **pgrx extension functions exclusively** for all database operations:

- `fhir_put(resource_type, resource_data)` - Creates FHIR resources
- `fhir_get(resource_type, resource_id)` - Retrieves FHIR resources
- `fhir_search(resource_type, param, op, value)` - Searches FHIR resources

All SQL operations go through these Rust-based PostgreSQL extension functions, providing:
- Type safety
- Better performance
- Consistent data handling
- Native Rust integration with PostgreSQL

## Local Development

For local development without Docker:

1. Install PostgreSQL 16 and development headers
2. Install Rust and cargo-pgrx:
   ```bash
   cargo install --locked cargo-pgrx --version 0.16.1
   cargo pgrx init --pg16 $(which pg_config)
   ```

3. Build and install the pgrx extension:
   ```bash
   cd db
   cargo pgrx install --release --pg-config $(which pg_config)
   ```

4. Initialize the database:
   ```bash
   createdb fhir_db
   psql fhir_db < db/sql/init.sql
   ```

5. Run the server:
   ```bash
   cargo run --release -p fhir-server
   ```

## Testing

```bash
# Run all tests
DATABASE_URL=postgresql://postgres:password@localhost:5432/fhir_db cargo test

# Run database tests only
cargo test --lib database::tests

# Run handler tests
cargo test --lib handlers::tests

# Run model tests
cargo test --lib models::tests

# Test the API (after server is running)
./test-api.sh
```

## API Endpoints

Once running on http://localhost:3000:

### Create a Patient
```bash
curl -X POST http://localhost:3000/fhir/Patient \
  -H "Content-Type: application/fhir+json" \
  -d '{"resourceType": "Patient", "name": [{"family": "Gauß", "given": ["Carl"]}], "gender": "male", "birthDate": "1990-01-01"}'
```

### Get a Patient
```bash
curl http://localhost:3000/fhir/Patient/{id}
```

### Get Patient History
```bash
curl http://localhost:3000/fhir/Patient/{id}/_history
```
Returns a Bundle with all versions of the patient resource, including timestamps and modifications.

### Example Error (OperationOutcome)
When the server returns an error, it responds with an `OperationOutcome` containing richer details (code, diagnostics, and location):

```json
{
   "resourceType": "OperationOutcome",
   "issue": [
      {
         "severity": "error",
         "code": "not-found",
         "diagnostics": "Patient with id 12345 not found",
         "location": ["Patient/12345"]
      }
   ]
}
```

### Search Patients
```bash
# Search by name
curl "http://localhost:3000/fhir/Patient?name=doe"

# Search by gender
curl "http://localhost:3000/fhir/Patient?gender=male"

# Search with pagination
curl "http://localhost:3000/fhir/Patient?_count=10&_offset=0"
```

## Technology Stack

- **PostgreSQL 16**: PostgreSQL 16 with JSONB and full-text search
- **Axum**: Modern async web framework for HTTP
- **SQLx**: Async SQL toolkit with compile-time verification
- **Tokio**: Async runtime
- **serde_json**: JSON serialization with JSONB support
- **FHIR v4.0.1**: Full healthcare data standards compliance

## Test Coverage

The server includes comprehensive unit and integration tests:

- **Database module (13 tests)**: Patient CRUD, search, pagination, versioning
- **Models module (9 tests)**: Serialization, deserialization, FHIR compliance
- **Handlers module (8 tests)**: HTTP handlers, status codes, error handling

Total: 30+ tests covering all core functionality

## Documentation

For complete setup and troubleshooting documentation, see:
- **README.md** - Project overview and API examples
