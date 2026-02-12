#!/bin/bash

set -e

echo "Running FHIR Server Tests..."

# Install the PostgreSQL extension first
echo "Installing PostgreSQL extension..."
cd db
sudo env HOME=$HOME PATH=$PATH RUSTUP_TOOLCHAIN=stable cargo pgrx install --pg-config /usr/bin/pg_config --release
cd ..

# Test the server
echo "Testing Axum server..."
cd server
cargo test
cd ..

echo "All tests passed!"