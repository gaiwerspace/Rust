# Standalone Dockerfile for the FHIR server
FROM rust:latest as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy server source code
COPY Cargo.toml ./
COPY src/ ./src/

# Build the server
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the built server
COPY --from=builder /app/target/release/fhir-server /usr/local/bin/fhir-server

EXPOSE 3000

CMD ["fhir-server"]