help:
    cargo run -- -h
dev:
    watchexec -r -e rs cargo run
doc:
    watchexec -r -e rs cargo doc --open
swagger:
    watchexec -r -e rs cargo r -- --with-swagger-ui
redoc:
    watchexec -r -e rs cargo r -- --with-redoc  --with-swagger-ui
scalar:
    watchexec -r -e rs cargo r -- --with-scalar --with-swagger-ui
rapidoc:
    watchexec -r -e rs cargo r -- --with-rapidoc --with-swagger-ui
run:
    cargo run

# Test commands
test:
    cargo test
test-integration:
    cargo test --test integration_tests
test-health:
    cargo test --test health_tests
test-word-api:
    cargo test --test word_tests
test-admin:
    cargo test --test admin_tests
test-config:
    cargo test --test config_tests
test-all:
    cargo test --tests
test-verbose:
    cargo test --tests -- --nocapture
test-parallel:
    cargo test --tests --test-threads=1
test-watch:
    watchexec -e rs cargo test --tests
