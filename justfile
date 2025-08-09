# Workspace-level justfile for random-word-api

# Show available commands
default:
    @just --list

# Build the entire workspace
build:
    cargo build

# Check the entire workspace
check:
    cargo check

# Test the entire workspace
test:
    cargo test

# Clean the entire workspace
clean:
    cargo clean

# Format all code in the workspace
fmt:
    cargo fmt --all

# Run clippy on the entire workspace
clippy:
    cargo clippy --all-targets --all-features

# Fix all auto-fixable issues
fix:
    cargo fix --all-targets --all-features --allow-dirty

# Run the API server
run *args:
    cargo run --bin word-api-axum -- {{args}}

# Run the API server in development mode with auto-reload
dev:
    watchexec -e rs cargo run --bin word-api-axum

# Run the API server with Swagger UI
swag:
    watchexec -e rs cargo run --bin word-api-axum -- --with-swagger-ui

# Generate API configuration file
gen-config:
    cargo run --bin word-api-axum -- gen-config

# Generate environment file
gen-env:
    cargo run --bin word-api-axum -- gen-env-file

# Run API-specific commands
api *args:
    cd word-api-axum && just {{args}}

# Install development dependencies
install-deps:
    cargo install watchexec-cli

# Run all quality checks (format, clippy, test)
ci: fmt clippy test

# Update all dependencies
update:
    cargo update

# Show workspace information
info:
    @echo "=== Workspace Information ==="
    @cargo tree --workspace
    @echo ""
    @echo "=== Workspace Members ==="
    @cargo metadata --format-version 1 | jq -r '.workspace_members[]'

# Validate workspace configuration
validate:
    @echo "=== Validating Workspace ==="
    @echo "Checking workspace structure..."
    @test -f Cargo.toml || (echo "❌ No workspace Cargo.toml found" && exit 1)
    @echo "✅ Workspace Cargo.toml exists"
    @grep -q "\[workspace\]" Cargo.toml || (echo "❌ No [workspace] section found" && exit 1)
    @echo "✅ Workspace section found"
    @echo "Checking workspace members can build..."
    @cargo check --workspace > /dev/null 2>&1 || (echo "❌ Workspace build failed" && exit 1)
    @echo "✅ All workspace members build successfully"
    @echo "Checking workspace dependencies..."
    @cargo tree --workspace --duplicates | grep -q "duplicate" && echo "⚠️  Duplicate dependencies found" || echo "✅ No duplicate dependencies"
    @echo "=== Workspace validation complete ==="
