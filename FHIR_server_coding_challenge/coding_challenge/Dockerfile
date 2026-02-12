# Simple Dockerfile for the FHIR server
FROM rust:latest AS builder

# Install build dependencies including PostgreSQL dev packages
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    postgresql-16 \
    postgresql-server-dev-16 \
    build-essential \
    libclang-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy all source code (workspace setup requires all members)
COPY server/ ./server/
COPY db/ ./db/
COPY Cargo.toml ./

# Install cargo-pgrx (match version from your error: 0.16.1)
RUN cargo install --locked cargo-pgrx --version 0.16.1

# Initialize pgrx with PostgreSQL 16 configuration
RUN cargo pgrx init --pg16=/usr/lib/postgresql/16/bin/pg_config

# Build the server from workspace root
RUN cargo build --release --bin fhir-server

# Build and install the FHIR extension
RUN cargo pgrx install --release --pg-config /usr/lib/postgresql/16/bin/pg_config --package fhir-extension

# Runtime stage
FROM postgres:16

# Install runtime dependencies including PostgreSQL runtime
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    postgresql-16 \
    && rm -rf /var/lib/apt/lists/*

# Copy the built server
COPY --from=builder /app/target/release/fhir-server /usr/local/bin/fhir-server

# Copy the installed PostgreSQL extension
COPY --from=builder /usr/share/postgresql/16/extension/fhir* /usr/share/postgresql/16/extension/
COPY --from=builder /usr/lib/postgresql/16/lib/fhir* /usr/lib/postgresql/16/lib/

EXPOSE 3000

CMD ["fhir-server"]
