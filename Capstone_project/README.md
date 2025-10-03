# Project: Rust & React Full-Stack Platform for News Article Management and Analysis with AI

### Objective

Create a platform for parsing and analyzing informational news articles.

### Core Deliverables

- **Vite:** New kind of build tool for frontend web development.
- **Rust API:** Lightweight and secure library for building backend APIs in Rust.
- **React Dashboard:** Modern web interface for server management.
- **Ollama**: open-source software that allows execution of powerful language models directly on local computer

### Environment Setup

Local instance on Ubuntu 24.04 or in Docker.
### Rust API (Axum + PostgreSQL/MySQL)

- **Architecture:** Vite, PostgreSQL/MySQL persistence (SQLx).

### React Dashboard (Vite SPA)

- **Stack:** React with Vite, modern UI component library

### Migration Utility (Rust CLI)

- **Direct Database Extraction:** The utility will connect directly to the  PostgreSQL/MySQL database to extract  information about posts.

### Full-Stack Quality and Validation

To validate the success of this build, a comprehensive test and benchmark suite covering both the frontend and backend is required.

- **Backend Testing:**
    - **Unit Tests:** Primarily for the migration's transformation logic.
    - **Integration Tests:** To verify the core API endpoints, including authentication, error handling, and successful orchestration of a server creation request.
- **Frontend Testing:**
    - **Component Tests:** Basic tests (e.g., using Vite/React Testing Library) must be included to verify the behavior of critical UI components like the login form and the server dashboard display.
- **Professional Performance Benchmarking:**
    - **API Performance:** JWT authentication throughput, database query speeds, async task processing latency, and memory usage profiling
    - **Migration Performance:** transformation speed, bulk insert rates, memory efficiency, and idempotency validation
    - **Frontend Performance:** React component render times, API response handling, and bundle size optimization

### Extended Technology & Protocol Stack

This stack includes the technologies explicitly mentioned and those implicitly required to successfully complete the project.

**Core Application & Language**

- **Rust:** The primary programming language for the API and migration utility.
- **Tokio:** The asynchronous runtime.
- **Vite:**  Fast frontend build tool powering the next generation of web applications.
- **SQLx:** The asynchronous SQL toolkit for Rust, used for interacting with PostgreSQL.
- **Serde:** The framework for serializing and deserializing Rust data structures, primarily for JSON API payloads.
- **Clap:** A command-line argument parser for building the standalone migration utility.
- **Tracing:** A framework for instrumenting Rust programs to collect structured, event-based diagnostic information.
- **Ollama**:  Ollama, an innovative open-source software that allows execution of powerful language models directly on local computer.

**Database Systems**

- **PostgreSQL/MySQL:** The target database for the new, modern application.
- **SQL:** Proficiency in both PostgreSQL and MySQL dialects is required.

**Web & API Technologies**

- **RESTful API Design:** Principles for creating clean, predictable web APIs.
- **HTTP:** Core understanding of methods (GET, POST), status codes (e.g., 200 OK, 202 Accepted, 401 Unauthorized), and headers.
- **JSON (JavaScript Object Notation):** The data format for API communication.
- **JWT (JSON Web Tokens):** The standard for stateless API authentication. Crates like `jsonwebtoken` would be used.

**Development, Tooling & Design Patterns**

- **Docker & Docker Compose:** For creating a reproducible local development environment that includes the Linux, PostgreSQL, and the new Rust API.
- **Cargo:** Rust's build system and package manager.
- **Git & GitHub/GitLab:** For version control.
- **`sqlx-cli`:** A command-line utility for managing database migrations (creating, applying, reverting).
---
