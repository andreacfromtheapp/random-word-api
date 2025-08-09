# Test Suite Documentation

## Overview

This test suite provides comprehensive coverage for the Random Word API, focusing
on clean, maintainable, and reliable integration testing. The suite has been
refactored from ~7,000 lines down to ~2,000 lines while maintaining excellent
coverage and adding performance monitoring.

## Test Structure

### Test Files

- `basic_test.rs` - Core functionality unit tests (20 tests)
- `config_tests.rs` - Configuration validation tests
- `admin_tests.rs` - Admin API endpoint tests
- `health_tests.rs` - Health check endpoint tests
- `word_tests.rs` - Public word API endpoint tests
- `performance_tests.rs` - Performance monitoring and reliability tests

### Helper Modules

- `helpers/mod.rs` - Core test utilities and setup functions
- `helpers/database.rs` - Database operations and performance monitoring
- `helpers/fixtures.rs` - Test data fixtures and factories

## Test Patterns and Guidelines

### Database Testing

#### Database Setup

```rust
// For tests that don't need direct DB access
let (server, _temp_file) = create_test_server().await?;

// For tests that need direct database operations
let (server, _temp_file, pool) = create_test_server_with_pool().await?;
```

#### Test Data Isolation

- Each test uses unique suffixes to avoid conflicts: `populate_test_data(&pool, "1").await?`
- Temporary databases are automatically cleaned up
- Use `database::cleanup_test_data()` for explicit cleanup if needed

#### Performance Monitoring

```rust
let (result, metrics) = measure_test_performance("operation_name", async_operation()).await?;
assert_test_performance(&metrics, performance_thresholds::DATABASE_OPERATION);
```

### API Testing

#### Standard Test Structure

```rust
#[tokio::test]
#[serial] // Use for tests that modify database state
async fn test_endpoint() -> Result<()> {
    let (server, _temp_file) = create_test_server().await?;

    let response = server.get("/endpoint").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    Ok(())
}
```

#### Response Validation

```rust
let json: serde_json::Value = response.json();
assert!(json.is_array(), "Public API returns arrays");
assert!(!json.as_array().unwrap().is_empty(), "Should have data");
```

### Error Testing

```rust
let response = server.get("/invalid/endpoint").await;
assert!(response.status_code() >= StatusCode::BAD_REQUEST);
```

## Performance Standards

### Timing Thresholds

- Database operations: ≤ 100ms
- API requests: ≤ 500ms
- Test setup: ≤ 200ms
- Bulk operations: ≤ 1000ms

### Performance Testing

```rust
let (result, metrics) = measure_test_performance("db_query", count_words(&pool)).await?;
assert_test_performance(&metrics, performance_thresholds::DATABASE_OPERATION);
```

## Reliability Features

### Retry Logic

```rust
let result = reliability::retry_operation(|| {
    Box::pin(potentially_flaky_operation())
}, 3).await?;
```

### Health Checks

```rust
reliability::validate_db_health(&pool).await?;
```

## Test Data Management

### Fixtures

Use predefined fixtures for consistent test data:

```rust
use helpers::fixtures::WordFixtures;

let nouns = WordFixtures::nouns();
let all_words = WordFixtures::all();
```

### Factories

Create dynamic test data:

```rust
use helpers::fixtures::WordFactory;

let word = WordFactory::create_with_suffix("test", "noun", "123");
let invalid_word = WordFactory::create_invalid("word");
```

### JSON Fixtures

```rust
use helpers::fixtures::JsonFixtures;

let valid_request = JsonFixtures::valid_word_request();
let incomplete_request = JsonFixtures::incomplete_word_request();
```

## Running Tests

### All Tests

```bash
cargo test
```

### Specific Test File

```bash
cargo test --test word_tests
```

### Single Test

```bash
cargo test test_random_word_endpoint
```

### Test Performance Monitoring

Tests automatically report timing for operations that exceed thresholds.

## Best Practices

### Do's

- Use unique suffixes for test data to avoid conflicts
- Apply `#[serial]` attribute for tests that modify database state
- Use performance monitoring for critical operations
- Validate both success and error scenarios
- Use meaningful test names that describe the scenario
- Add context to assertions with descriptive error messages

### Don'ts

- Don't hardcode specific response values that might vary
- Don't create overly complex test scenarios
- Don't skip cleanup for integration tests
- Don't ignore performance regressions
- Don't test implementation details, focus on behavior

### Error Handling

- Always use `Result<()>` return type for async tests
- Use `.context()` for meaningful error messages
- Test both valid and invalid inputs
- Validate error response formats

## Coverage Goals

### Current Coverage (85 tests)

- ✅ Core API endpoints (public and admin)
- ✅ Database operations
- ✅ Configuration validation
- ✅ Health checks
- ✅ Basic error scenarios

### Areas for Future Expansion

- Edge cases for input validation
- Concurrent operation testing
- Rate limiting scenarios
- Complex error recovery
- Load testing scenarios

## Maintenance

### Adding New Tests

1. Follow existing patterns in similar test files
2. Use appropriate helper functions
3. Add performance monitoring for new critical operations
4. Include both positive and negative test cases
5. Update this documentation if introducing new patterns

### Performance Regression Detection

- Monitor test execution times
- Performance assertions will fail if operations exceed thresholds
- Use `measure_test_performance()` for new critical operations

### Database Schema Changes

- Update test fixtures in `helpers/fixtures.rs`
- Verify migration compatibility in test setup
- Add new validation tests for schema changes

## Troubleshooting

### Common Issues

**Tests fail with database errors:**

- Ensure migrations are applied correctly
- Check that temporary database setup is working
- Verify database pool initialization

**Performance test failures:**

- Check if operations are taking longer than expected
- Consider if thresholds need adjustment for your environment
- Look for resource contention in parallel tests

**Flaky tests:**

- Use `#[serial]` for tests that share state
- Use unique test data suffixes
- Consider adding retry logic for network-dependent operations

### Debug Mode

```bash
RUST_LOG=debug cargo test -- --nocapture
```

## Test Metrics Summary

- **Total Tests**: 85 (20 unit + 65 integration)
- **Test Files**: 5 main test files + helper modules
- **Code Coverage**: High coverage of API endpoints and core functionality
- **Performance**: All tests complete within defined thresholds
- **Reliability**: Zero flaky tests with proper isolation
- **Maintainability**: Clean, documented, standardized patterns
