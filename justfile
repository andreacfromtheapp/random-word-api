# Workspace-level justfile for random-word-api

# Show available commands
default:
    @just --list

# Run the API server
run *args:
    cargo run --bin word-api-axum -- {{args}}

# Run the API server in development mode with auto-reload
dev:
    watchexec -r -e rs cargo run --bin word-api-axum

# Run the API server with Swagger UI
swagger:
    watchexec -r -e rs cargo run --bin word-api-axum -- --with-swagger-ui
# Run the API server with Redoc
redoc:
    watchexec -r -e rs cargo run --bin word-api-axum -- --with-redoc  --with-swagger-ui
# Run the API server with Scalar
scalar:
    watchexec -r -e rs cargo run --bin word-api-axum -- --with-scalar  --with-swagger-ui
# Run the API server with RapiDoc
rapidoc:
    watchexec -r -e rs cargo run --bin word-api-axum -- --with-rapidoc  --with-swagger-ui

# Generate API configuration file
gen-config:
    cargo run --bin word-api-axum -- gen-config

# Generate environment file
gen-env:
    cargo run --bin word-api-axum -- gen-env-file

# Install development dependencies
install-deps:
    cargo install watchexec-cli

# Show workspace information
info:
    @echo "=== Workspace Information ==="
    @cargo tree --workspace
    @echo ""
    @echo "=== Workspace Members ==="
    @cargo metadata --format-version 1 | jq -r '.workspace_members[]'

