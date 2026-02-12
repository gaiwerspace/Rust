FROM postgres:16

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    libclang-dev \
    clang \
    postgresql-server-dev-16 \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install cargo-pgrx
RUN cargo install --locked cargo-pgrx --version 0.16.1

# Copy the extension source
WORKDIR /build/fhir-extension
COPY db/Cargo.toml ./Cargo.toml
COPY db/src ./src
COPY db/fhir_extension.control ./fhir_extension.control

# Create a standalone Cargo.toml (remove workspace dependencies)
RUN sed -i 's/{ workspace = true }/{ version = "1.0", features = ["derive"] }/g' Cargo.toml && \
    sed -i 's/serde = .*/serde = { version = "1.0", features = ["derive"] }/' Cargo.toml && \
    sed -i 's/serde_json = .*/serde_json = "1.0"/' Cargo.toml && \
    sed -i 's/uuid = .*/uuid = { version = "1.0", features = ["v4", "serde"] }/' Cargo.toml && \
    sed -i 's/chrono = .*/chrono = { version = "0.4", features = ["serde"] }/' Cargo.toml

# Initialize pgrx with system PostgreSQL
RUN cargo pgrx init --pg16 $(which pg_config)

# Build pgrx_embed binary (required for schema generation during install)
RUN cargo build --release --bin pgrx_embed_fhir_extension --no-default-features --features pg16

# Ensure cargo-pgrx can find pgrx_embed (it searches on PATH)
RUN install -m 0755 target/release/pgrx_embed_fhir_extension /usr/local/bin/pgrx_embed_fhir_extension

# Build and install the extension
RUN cargo pgrx install --release --pg-config $(which pg_config)

# Switch back to postgres user
USER postgres

# Set the default command
CMD ["postgres"]
