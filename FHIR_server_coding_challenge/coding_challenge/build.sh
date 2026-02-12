#!/bin/bash

set -e

echo "Building FHIR Server with PostgreSQL Extension..."

# Build the PostgreSQL extension
echo "Building PostgreSQL extension..."
cd db
cargo pgrx package --release
cd ..

# Build the server
echo "Building Axum server..."
cd server
cargo build --release
cd ..

echo "Build complete!"
echo ""
echo "To run the server:"
echo "1. Start PostgreSQL and create the database"
echo "2. Install the extension: cd db && cargo pgrx install --release"
echo "3. Initialize the database: psql fhir_db < db/sql/init.sql"
echo "4. Run the server: cd server && cargo run --release"
echo ""
echo "Or use Docker Compose: docker-compose up --build"