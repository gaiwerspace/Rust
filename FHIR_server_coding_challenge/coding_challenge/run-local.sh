#!/bin/bash

set -e

echo "Starting FHIR Server locally..."

# Check if PostgreSQL is running
if ! pg_isready -h localhost -p 5432 >/dev/null 2>&1; then
    echo "PostgreSQL is not running. Please start PostgreSQL first."
    echo "You can use: brew services start postgresql"
    echo "Or: docker run --name postgres -e POSTGRES_PASSWORD=password -e POSTGRES_DB=fhir_db -p 5432:5432 -d postgres:16"
    exit 1
fi

# Create database if it doesn't exist
createdb fhir_db 2>/dev/null || echo "Database fhir_db already exists"

# Initialize the database
echo "Initializing database..."
psql fhir_db < db/sql/init.sql

# Build and run the server
echo "Building and starting the server..."
cd server
export DATABASE_URL="postgresql://postgres:password@localhost:5432/fhir_db"
export RUST_LOG=info
cargo run --release