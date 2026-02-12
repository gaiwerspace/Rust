#!/bin/bash
# Setup script for FHIR Patient Server

set -e

echo "=========================================="
echo "FHIR Patient Server Setup Script"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check prerequisites
echo -e "\n${YELLOW}Checking prerequisites...${NC}"

# Check PostgreSQL
if ! command -v psql &> /dev/null; then
    echo -e "${RED}PostgreSQL client not found. Please install PostgreSQL.${NC}"
    exit 1
fi

# Check Rust
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Rust not found. Please install Rust from https://rustup.rs/${NC}"
    exit 1
fi

echo -e "${GREEN}✓ All prerequisites found${NC}"

# Get database configuration
echo -e "\n${YELLOW}Database Configuration${NC}"
DB_USER=${POSTGRES_USER:-postgres}
DB_PASSWORD=${POSTGRES_PASSWORD:-password}
DB_HOST=${POSTGRES_HOST:-localhost}
DB_PORT=${POSTGRES_PORT:-5432}
DB_NAME=${FHIR_DB_NAME:-fhir_db}

echo "  User: $DB_USER"
echo "  Host: $DB_HOST"
echo "  Port: $DB_PORT"
echo "  Database: $DB_NAME"

# Test database connection
echo -e "\n${YELLOW}Testing database connection...${NC}"
if ! PGPASSWORD=$DB_PASSWORD psql -U $DB_USER -h $DB_HOST -p $DB_PORT -c "SELECT version();" &> /dev/null; then
    echo -e "${RED}Cannot connect to PostgreSQL at $DB_HOST:$DB_PORT${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Database connection successful${NC}"

# Create database if it doesn't exist
echo -e "\n${YELLOW}Creating database...${NC}"
if PGPASSWORD=$DB_PASSWORD psql -U $DB_USER -h $DB_HOST -p $DB_PORT -tc "SELECT 1 FROM pg_database WHERE datname = '$DB_NAME'" | grep -q 1; then
    echo "  Database '$DB_NAME' already exists"
else
    echo "  Creating database '$DB_NAME'..."
    PGPASSWORD=$DB_PASSWORD psql -U $DB_USER -h $DB_HOST -p $DB_PORT -c "CREATE DATABASE $DB_NAME;"
    echo -e "${GREEN}✓ Database created${NC}"
fi

# Run migrations
echo -e "\n${YELLOW}Running migrations...${NC}"
if [ -f "migrations/run_migrations.sql" ]; then
    echo "  Running: migrations/run_migrations.sql"
    PGPASSWORD=$DB_PASSWORD psql -U $DB_USER -h $DB_HOST -p $DB_PORT -d $DB_NAME -f migrations/run_migrations.sql
    echo -e "${GREEN}✓ Migrations completed${NC}"
elif [ -f "migrations/001_initial_schema.sql" ]; then
    echo "  Running migration files in sequence..."
    for migration in migrations/001_initial_schema.sql migrations/002_add_search_functions.sql migrations/002_fhir_extension_functions.sql migrations/003_fhir_search_helpers.sql; do
        if [ -f "$migration" ]; then
            echo "  Running: $migration"
            PGPASSWORD=$DB_PASSWORD psql -U $DB_USER -h $DB_HOST -p $DB_PORT -d $DB_NAME -f "$migration"
        fi
    done
    echo -e "${GREEN}✓ Migrations completed${NC}"
else
    echo -e "${RED}Migration files not found in migrations/ directory${NC}"
    exit 1
fi

# Set DATABASE_URL for application
export DATABASE_URL="postgresql://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME"
echo -e "\n${YELLOW}DATABASE_URL set to:${NC}"
echo "$DATABASE_URL"

# Verify schema
echo -e "\n${YELLOW}Verifying schema...${NC}"
PGPASSWORD=$DB_PASSWORD psql -U $DB_USER -h $DB_HOST -p $DB_PORT -d $DB_NAME << EOF
\echo 'Checking FHIR schema...'
SELECT schema_name FROM information_schema.schemata WHERE schema_name = 'fhir';
\echo 'Checking functions...'
SELECT COUNT(*) as function_count FROM information_schema.routines WHERE routine_schema = 'fhir';
\echo 'Checking tables...'
SELECT COUNT(*) as table_count FROM information_schema.tables WHERE table_schema = 'fhir';
EOF

echo -e "${GREEN}✓ Schema verification complete${NC}"

# Compile Rust code
echo -e "\n${YELLOW}Building Rust project...${NC}"
cargo build --release 2>&1 | tail -5
echo -e "${GREEN}✓ Build complete${NC}"

# Summary
echo -e "\n${GREEN}=========================================="
echo "Setup Complete!"
echo "=========================================${NC}"

echo -e "\n${YELLOW}Next steps:${NC}"
echo "1. Start the server:"
echo "   DATABASE_URL=$DATABASE_URL cargo run --bin fhir-server"
echo ""
echo " Or with environment file:"
echo "   echo 'DATABASE_URL=$DATABASE_URL' > .env"
echo "   cargo run --bin fhir-server"
echo ""
echo "2. Test the API:"
echo "   curl http://localhost:3000/fhir/Patient"
echo ""
echo "3. Run tests:"
echo "   DATABASE_URL=$DATABASE_URL cargo test"

echo -e "\n${YELLOW}For more information, see SETUP.md${NC}"
