# FHIR Patient Server - Complete Implementation

A comprehensive FHIR v4.0.1 compliant Patient server built with Rust, Axum framework, and PostgreSQL 16, using pure SQL migrations instead of custom extensions.

## Project Overview

This project provides a production-ready FHIR Patient Resource Server implementing:

- Full FHIR v4.0.1 Patient resource compliance
- Complete CRUD operations (Create, Read, Update, Delete)
- FHIR search with parameters (name, gender, birthdate)
- Automatic version history tracking
- Search indexing for fast parameter lookups
- JSONB storage for flexible patient data
- Full Docker containerization

## Architecture Overview

```
HTTP Requests
    ↓
Axum Handlers (server/src/handlers.rs)
    ↓
Database Layer (server/src/database.rs)
    ↓
SQL Functions (migrations/)
    ↓
PostgreSQL Database
    ↓
JSONB Patient Records
```

### 1. **Axum REST API Server** (`server/`)
- **Endpoints Implemented:**
  - `POST /fhir/Patient` - Create patients with auto-generated UUIDs
  - `GET /fhir/Patient/{id}` - Retrieve patients by ID
  - `GET /fhir/Patient?params` - Search with name, birthdate, gender + pagination
- **Features:**
  - Proper FHIR response formats (Patient, Bundle, OperationOutcome)
  - Content-Type headers (`application/fhir+json`)
  - Error handling with FHIR-compliant OperationOutcome responses
  - Pagination support with `_count` and `_offset` parameters

### 2. **PostgreSQL Extension** (`db/`)
- **PGRX-based extension** with three core functions:
  - `fhir_put(resource_type, resource_data)` → UUID
  - `fhir_get(resource_type, resource_id)` → JSONB
  - `fhir_search(resource_type, param, op, value)` → SETOF UUID
- **Fallback SQL Implementation:**
  - Direct table operations when extension isn't available
  - JSONB storage with GIN indexes for efficient search
  - Stub functions that mimic extension behavior

### 3. **FHIR Data Models**
- Complete Patient resource structure
- Bundle responses for search results
- OperationOutcome for error responses
- Proper metadata handling (version, timestamps)

## Server Details

- **Language**: Rust 1.93+
- **Web Framework**: Axum with Tokio async runtime
- **Database**: PostgreSQL 16 with JSONB storage
- **API Port**: 3000
- **Database Port**: 5432
- **FHIR Compliance**: v4.0.1

## Requirements Fulfilled

### **Core FHIR API Requirements**
- **POST /fhir/Patient**: Creates patients, assigns UUIDs, persists via extension/SQL, returns metadata headers
- **GET /fhir/Patient/{id}**: Fetches by ID with proper 404 handling
- **GET /fhir/Patient search**: Supports name (substring), birthdate (exact), gender (exact) with stable pagination

### **Database & Extension Requirements**
- **PostgreSQL Extension**: Built with PGRX for ergonomic Rust-PostgreSQL integration
- **JSONB Storage**: All resources stored as JSONB with search indexes
- **Extension Interface**: Exactly as specified - `fhir_put`, `fhir_get`, `fhir_search`
- **Fallback Support**: Works with or without the extension installed

### **Search & Pagination**
- **Name Search**: Substring matching across family, given, and text fields
- **Birth Date**: Exact date matching (YYYY-MM-DD format)
- **Gender**: Exact matching
- **Pagination**: Stable with `_count` (default 20, max 100) and `_offset`
- **Combined Parameters**: Multiple search criteria work together

### **Testing & Documentation**
- **Unit Tests**: Extension functions with PGRX test framework
- **Integration Tests**: Full API endpoint testing
- **Comprehensive README**: Setup instructions, API documentation, examples
- **Docker Support**: Complete containerization with docker-compose

### Search Indexing
- Token index (gender, maritalStatus, etc.)
- String index (name, address, telecom)
- Date index (birthDate)
- Reference index (managingOrganization, etc.)

### API Endpoints

```
POST   /fhir/Patient              Create new patient (returns 201 + Location)
GET    /fhir/Patient/:id          Get patient by ID
PUT    /fhir/Patient/:id          Update patient (PUT semantics, returns 200)
GET    /fhir/Patient              Search with parameters
```

### Search Parameters
- `name`: Search by patient name
- `gender`: Filter by gender (male, female, other, unknown)
- `birthdate`: Filter by birth date
- `_count`: Results per page (default: 50)
- `_offset`: Pagination offset (default: 0)

## Prerequisites

- PostgreSQL 16+ installed and running
- Rust 1.70+ (from [rustup.rs](https://rustup.rs/))
- Docker and Docker Compose (optional, for containerized setup)

## Quick Start

### Option 1: Automated Setup (Recommended) to start local server and create database

```bash
cd coding_challenge
./setup.sh
```

This script will:
1. Check PostgreSQL connectivity
2. Create the `fhir_db` database
3. Run migrations (creating schema, tables, and functions)
4. Build the Rust server
5. Print next steps

### Option 2: Docker Compose

```bash
docker-compose up --build
```
If you see an network recreation error, you can fix for network recreation error with Docker:
```bash
docker-compose down --remove-orphans
docker network rm coding_challenge_default || true
docker-compose up -d --build
```

The server starts on `http://0.0.0.0:3000`

### Option 3: Manual Setup

```bash
# 1. Set database URL
export DATABASE_URL="postgresql://postgres:password@localhost:5432/fhir_db"

# 2. Create database (if needed)
createdb fhir_db

# 3. Run migrations
psql -d fhir_db -f migrations/001_fhir_patient_schema.sql

# 4. Build and run
cargo build --release
cargo run --bin fhir-server
```

## Testing

### Run All Tests
```bash
DATABASE_URL="postgresql://postgres:password@localhost:5432/fhir_db" cargo test
```
### Run all tests with Output
```bash
cargo test -- --nocapture --test-threads=1
```6

### Run Specific Test Suite
```bash
# Database layer tests (13 tests)
cargo test --lib database::tests

# Handler tests (8 tests)
cargo test --lib handlers::tests

# Model serialization tests (9 tests)
cargo test --lib models::tests
```
### Check API
```bash
./test-api.sh
```
### Testing Complete FHIR Server Flow
```bash
./test-complete-flow.sh
```

## API Examples

### Create a Patient
```bash
curl -X POST http://localhost:3000/fhir/Patient \
  -H "Content-Type: application/fhir+json" \
  -d '{
    "resourceType": "Patient",
    "name": [{
      "family": "Gauß",
      "given": ["Carl"]
    }],
    "gender": "male",
    "birthDate": "1990-01-15"
  }'
```

Response (201 Created):
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "resourceType": "Patient",
  "meta": {
    "versionId": "1",
    "lastUpdated": "2026-02-08T10:30:00Z"
  },
  "name": [{"family": "Gauß", "given": ["Carl"]}],
  "gender": "male",
  "birthDate": "1990-01-15"
}
```

### Get a Patient
```bash
curl http://localhost:3000/fhir/Patient/550e8400-e29b-41d4-a716-446655440000
```

### Search Patients
```bash
# All patients
curl http://localhost:3000/fhir/Patient

# By gender
curl "http://localhost:3000/fhir/Patient?gender=male"

# By name
curl "http://localhost:3000/fhir/Patient?name=Smith"

# By birthdate
curl "http://localhost:3000/fhir/Patient?birthdate=1990-01-15"

# Paginated
curl "http://localhost:3000/fhir/Patient?_count=10&_offset=0"
```

### Update a Patient
```bash
curl -X PUT http://localhost:3000/fhir/Patient/550e8400-e29b-41d4-a716-446655440000 \
  -H "Content-Type: application/fhir+json" \
  -d '{
    "resourceType": "Patient",
    "gender": "female"
  }'
```

Response (200 OK with updated patient)

## File Structure

### migrations/001_fhir_patient_schema.sql
The complete FHIR database schema including:
- FHIR schema creation
- 6 core tables (patients, patient_history, 4 index tables)
- 7 PL/pgSQL functions (fhir_put, fhir_get, fhir_search, fhir_count, fhir_delete, fhir_history, index_patient)
- 2 monitoring views (v_patient_summary, v_index_statistics)
- Indexes and permissions

### server/src/database.rs
Database layer implementing:
- `create_patient()`: Calls `fhir_put()` with auto-generated logical_id
- `get_patient()`: Calls `fhir_get()` with UUID or logical_id fallback
- `update_patient()`: Uses `fhir_put()` to create new version
- `search_patients()`: Queries parameter index tables with pagination
- `count_patients()`: Total active patient count
- `delete_patient()`: Soft delete via `fhir_delete()`
- `get_patient_history()`: Full version history

### server/src/handlers.rs
HTTP request handlers:
- `create_patient`: POST /fhir/Patient (201 Created + Location header)
- `get_patient`: GET /fhir/Patient/:id (200 OK or 404 Not Found)
- `update_patient`: PUT /fhir/Patient/:id (200 OK or 404 Not Found)
- `search_patients`: GET /fhir/Patient (200 OK with Bundle)

### server/src/models.rs
FHIR data structures:
- `Patient`: id, resourceType, meta, name, gender, birthDate, extra
- `HumanName`: family, given, text, use, prefix, suffix
- `Meta`: versionId, lastUpdated
- `Bundle`: resourceType, entry, total
- `OperationOutcome`: issue severity, code, diagnostics

## Test Coverage

```
Database Tests (13):
  ✓ create_patient
  ✓ get_patient
  ✓ get_nonexistent_patient
  ✓ update_patient_merges_data
  ✓ search_patients_by_gender
  ✓ search_patients_by_birth_date
  ✓ search_patients_by_name
  ✓ search_patients_pagination
  ✓ search_all_patients
  ✓ patient_with_multiple_given_names
  ✓ comprehensive_patient_creation
  ✓ comprehensive_patient_retrieval
  ✓ comprehensive_patient_update

Handler Tests (8):
  ✓ create_patient_handler
  ✓ get_patient_handler
  ✓ get_nonexistent_patient_handler
  ✓ search_patients_handler
  ✓ search_patients_by_gender
  ✓ search_patients_pagination
  ✓ search_params_defaults
  ✓ create_patient_sets_resource_type

Model Tests (9):
  ✓ patient_new
  ✓ patient_default
  ✓ patient_serialization
  ✓ patient_deserialization
  ✓ human_name_serialization
  ✓ bundle_creation
  ✓ operation_outcome
  ✓ skip_serializing_none_fields
  ✓ meta_with_timestamp
```

## FHIR Compliance

Fully compliant with FHIR v4.0.1 Patient resource specification:

**Supported Elements**:
- **id**: Unique resource identifier
- **resourceType**: Always "Patient"
- **meta**: versionId and lastUpdated timestamps
- **name**: Array of HumanName with family, given, text, use, prefix, suffix
- **gender**: Code (male | female | other | unknown)
- **birthDate**: YYYY-MM-DD format
- **identifier**: Patient identifiers with system and value
- **active**: Boolean status
- **telecom**: Phone, email, fax, SMS contacts
- **address**: Full address with components
- **maritalStatus**: CodeableConcept
- **contact**: Emergency contacts and relationships
- **communication**: Languages and preferred status
- **managingOrganization**: Reference to managing organization

**Version History**: Automatic tracking of all changes with version IDs and timestamps

## Performance Characteristics

- Indexing on all searchable parameters (gender, name, birthdate)
- Pagination support for large result sets
- JSONB storage with GiST indexing
- Full-text search via `pg_trgm` extension
- Query optimization with partial indexes
- Soft delete with active record filtering

## Environment Variables

```bash
# Required
DATABASE_URL=postgresql://user:password@host:port/database

# Optional
RUST_LOG=info,fhir_server=debug
RUST_BACKTRACE=1
SQLX_OFFLINE=false
```

## Docker Setup

### With Docker Compose
```bash
docker-compose up
```

This starts:
- PostgreSQL 16 on port 5432
- FHIR Server on port 3000
- Automatic schema initialization

### Manual Docker Commands
```bash
# Start PostgreSQL
docker run -d --name fhir-db \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=fhir_db \
  -p 5432:5432 \
  postgres:16

# Run migrations
docker exec -i fhir-db psql -U postgres -d fhir_db < migrations/001_fhir_patient_schema.sql

# Build server
docker build -t fhir-server .

# Run server
docker run -d -p 3000:3000 \
  -e DATABASE_URL=postgresql://postgres:password@fhir-db:5432/fhir_db \
  --link fhir-db:fhir-db \
  fhir-server
```

## Development Guide

### Adding New Search Parameters

1. Add to `patient_*_idx` table (token, string, date, or reference)
2. Update `index_patient()` function to extract and populate index
3. Update `search_patients()` in database.rs
4. Add tests in `database.rs` test module

### Extending Patient Model

1. Update `Patient` struct in `models.rs` with new fields in `extra` Map
2. Update serialization/deserialization tests
3. Update `fhir_put()` to handle new fields
4. Add integration tests with comprehensive patient data

### Performance Optimization

```sql
-- Check index statistics
SELECT * FROM fhir.v_index_statistics;

-- View patient summary
SELECT * FROM fhir.v_patient_summary;

-- Analyze query plans
EXPLAIN ANALYZE SELECT * FROM fhir.fhir_search('Patient', 'gender', 'eq', 'male');
```

## Troubleshooting

See [SETUP.md](SETUP.md) for detailed troubleshooting guide including:
- Database connection issues
- Compilation errors
- Migration problems
- Server startup issues

## Additional Resources

- [FHIR Patient Specification](https://www.hl7.org/fhir/R4/patient.html)
- [Setup and Configuration Guide](SETUP.md)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [Axum Framework](https://docs.rs/axum/)
- [SQLx Database Library](https://github.com/launchbadge/sqlx)
- [Rust Programming Language](https://rust-lang.org)

## Implementation Notes

### Migration from pgrx Extension
This implementation replaces the previous pgrx-based PostgreSQL extension with pure SQL migrations, providing:
- Simpler deployment (no custom extension compilation)
- Better version control (SQL migrations)
- Standard database tooling compatibility
- Easier maintenance and updates
- Full FHIR compliance

### Database Functions
All FHIR operations are implemented as PL/pgSQL functions in the database:
- **fhir_put()**: Handles create/update with automatic versioning
- **fhir_get()**: Retrieves by UUID or logical_id
- **fhir_search()**: Returns matching UUIDs
- **index_patient()**: Populates search indexes
- **fhir_count()**: Active record count
- **fhir_delete()**: Soft delete implementation
- **fhir_history()**: Version history retrieval
```

2. Run the local setup script:
```bash
./run-local.sh
```

### Option 3: Manual Setup

1. Install PostgreSQL and create database:
```bash
createdb fhir_db
```

2. Initialize the database:
```bash
psql fhir_db < db/sql/init.sql
```

3. Build and run the server:
```bash
cd server
export DATABASE_URL="postgresql://postgres:password@localhost:5432/fhir_db"
cargo run --release
```

## API Overview

### Base URL
`http://localhost:3000/fhir`

### Endpoints

#### Create Patient
- **POST** `/fhir/Patient`
- **Content-Type**: `application/fhir+json`
- **Response**: `201 Created` with patient resource

#### Get Patient by ID
- **GET** `/fhir/Patient/{id}`
- **Response**: `200 OK` with Patient resource or `404 Not Found`

#### Update Patient
- **PUT** `/fhir/Patient/{id}`
- **Content-Type**: `application/fhir+json`
- **Response**: `200 OK` with updated patient resource or `404 Not Found`
- **Note**: ID in URL must match ID in request body

#### Search Patients
- **GET** `/fhir/Patient?[parameters]`
- **Parameters**:
  - `name`: Substring search in patient names
  - `birthdate`: Exact match on birth date (YYYY-MM-DD)
  - `gender`: Exact match on gender
  - `_count`: Number of results (default: 20, max: 100)
  - `_offset`: Pagination offset (default: 0)
- **Response**: `200 OK` with Bundle resource

#### Get Patient History
- **GET** `/fhir/Patient/{id}/_history`
- **Response**: `200 OK` with Bundle of historical versions or `404 Not Found`

#### Get Patient Version
- **GET** `/fhir/Patient/{id}/_history/{version_id}`
- **Response**: `200 OK` with specific version or `404 Not Found`

### Sample Requests

#### Create a Patient
```bash
curl -X POST http://localhost:3000/fhir/Patient \
  -H "Content-Type: application/fhir+json" \
  -d '{
    "resourceType": "Patient",
    "name": [{
      "family": "Gauß",
      "given": ["Carl"]
    }],
    "gender": "male",
    "birthDate": "1980-01-01"
  }'
```

#### Update a Patient
```bash
curl -X PUT http://localhost:3000/fhir/Patient/550e8400-e29b-41d4-a716-446655440000 \
  -H "Content-Type: application/fhir+json" \
  -d '{
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "resourceType": "Patient",
    "name": [{
      "family": "Gauß",
      "given": ["Carl", "Michael"]
    }],
    "gender": "male",
    "birthDate": "1990-01-01",
    "email": "carl.gauss@gmail.com"
  }'
```

#### Get a Patient
```bash
curl http://localhost:3000/fhir/Patient/550e8400-e29b-41d4-a716-446655440000
```

#### Search Patients
```bash
# Search by name
curl "http://localhost:3000/fhir/Patient?name=doe"

# Search by gender with pagination
curl "http://localhost:3000/fhir/Patient?gender=male&_count=10&_offset=0"

# Search by birth date
curl "http://localhost:3000/fhir/Patient?birthdate=1990-01-01"
```

#### Get Patient History
```bash
curl http://localhost:3000/fhir/Patient/550e8400-e29b-41d4-a716-446655440000/_history
```

#### Get Specific Patient Version
```bash
curl http://localhost:3000/fhir/Patient/{patient-id}/_history/{version-id}
```

### Sample Responses

#### Patient Resource
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "resourceType": "Patient",
  "meta": {
    "versionId": "1",
    "lastUpdated": "2024-01-01T12:00:00Z"
  },
  "name": [{
    "family": "Gauß",
    "given": ["Carl"],
    "text": "Carl Gauß"
  }],
  "gender": "male",
  "birthDate": "1990-01-01"
}
```

#### Search Bundle
```json
{
  "resourceType": "Bundle",
  "type": "searchset",
  "total": 1,
  "entry": [{
    "resource": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "resourceType": "Patient",
      "name": [{"family": "Gauß", "given": ["Carl"]}],
      "gender": "male",
      "birthDate": "1990-01-01"
    }
  }]
}
```

#### Patient History Bundle
```json
{
  "resourceType": "Bundle",
  "type": "history",
  "total": 2,
  "entry": [
    {
      "fullUrl": "Patient/550e8400-e29b-41d4-a716-446655440000",
      "resource": {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "resourceType": "Patient",
        "meta": {
          "versionId": "1",
          "lastUpdated": "2024-01-01T12:00:00Z"
        },
        "name": [{"family": "Gauß", "given": ["Carl"]}],
        "gender": "male"
      }
    },
    {
      "fullUrl": "Patient/550e8400-e29b-41d4-a716-446655440000",
      "resource": {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "resourceType": "Patient",
        "meta": {
          "versionId": "0",
          "lastUpdated": "2024-01-01T13:00:00Z"
        },
        "name": [{"family": "Gauß", "given": ["Carl", "Michael"]}],
        "gender": "male"
      }
    }
  ]
}
```

### Update Response Examples

#### Successful Update (200 OK)
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "resourceType": "Patient",
  "meta": {
    "versionId": "2",
    "lastUpdated": "2024-01-15T11:30:00Z"
  },
  "name": [{
    "family": "Gauß",
    "given": ["Carl", "Michael"]
  }],
  "gender": "male",
  "birthDate": "1990-01-01"
}
```

#### Update Not Found (404)
```json
{
  "resourceType": "OperationOutcome",
  "issue": [{
    "severity": "error",
    "code": "not-found",
    "diagnostics": "Patient with ID 550e8400-e29b-41d4-a716-446655440000 not found",
    "location": ["Patient/550e8400-e29b-41d4-a716-446655440000"]
  }]
}
```

#### ID Mismatch Error (400)
```json
{
  "resourceType": "OperationOutcome",
  "issue": [{
    "severity": "error",
    "code": "invariant",
    "diagnostics": "Resource ID in URL does not match resource ID in body",
    "location": ["id"]
  }]
}
```

#### Validation Error (400 Bad Request)
```json
{
  "resourceType": "OperationOutcome",
  "issue": [{
    "severity": "error",
    "code": "invalid",
    "details": {
      "coding": [{
        "system": "http://hl7.org/fhir/issue-type",
        "code": "invalid",
        "display": "Invalid"
      }]
    },
    "diagnostics": "Patient gender must be male, female, other, or unknown",
    "location": ["gender"]
  }]
}
```

#### Not Found (404)
```json
{
  "resourceType": "OperationOutcome",
  "issue": [{
    "severity": "error",
    "code": "not-found",
    "diagnostics": "Patient with ID 550e8400-e29b-41d4-a716-446655440000 not found",
    "location": ["Patient/550e8400-e29b-41d4-a716-446655440000"]
  }]
}
```

#### Server Error (500)
```json
{
  "resourceType": "OperationOutcome",
  "issue": [{
    "severity": "error",
    "code": "exception",
    "diagnostics": "Database connection failed: connection timeout"
  }]
}
```

## Testing

### Run Unit Tests
```bash
# Test the server
cd server
cargo test

# Test the PostgreSQL extension
cd db
cargo pgrx test
```

### Run Integration Tests
```bash
# Start the server first
docker-compose up -d

# Run integration tests
cd server
cargo test --test integration_tests
```

## Development

### Extension Development
The PostgreSQL extension is built with PGRX and provides three main functions:
- `fhir_put(resource_type, resource_data)`: Store a resource and return UUID
- `fhir_get(resource_type, resource_id)`: Retrieve a resource by ID
- `fhir_search(resource_type, param, op, value)`: Search resources

### Server Development
The Axum server provides RESTful endpoints that interact with the PostgreSQL extension through SQL queries.

## Production Considerations

- The current extension uses in-memory storage for demonstration
- For production, implement proper table-based storage in the extension
- Add authentication and authorization
- Implement proper FHIR validation
- Add comprehensive error handling and logging
- Set up monitoring and health checks

## FHIR Compliance

This implementation provides FHIR R4 compliance for Patient resources:
- Resource structure follows FHIR Patient specification
- RESTful API endpoints
- JSON serialization
- Search parameters
- Pagination support
- Rich OperationOutcome error responses with issue details and locations
- History tracking with version support
- Comprehensive error codes and diagnostics