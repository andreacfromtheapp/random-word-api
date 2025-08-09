# Test Suite Maintenance Guide

## Overview

This guide provides instructions for maintaining the Random Word API test suite,
troubleshooting common issues, and ensuring long-term reliability of the testing
infrastructure.

## Maintenance Tasks

### Daily/Continuous Integration

- Monitor test execution times for performance regressions
- Ensure all tests pass consistently
- Check for new compiler warnings or clippy suggestions
- Verify test coverage reports

### Weekly

- Review test performance metrics for trends
- Check for flaky test patterns
- Update dependencies if needed
- Review and clean up any accumulated test data

### Monthly

- Review test documentation for accuracy
- Assess test coverage gaps
- Performance threshold review and adjustment
- Dependency security audit

## Performance Monitoring

### Performance Baselines

Current performance expectations:

- Database operations: ≤ 100ms
- API requests: ≤ 500ms
- Test setup: ≤ 200ms
- Bulk operations: ≤ 1000ms
- Health checks: ≤ 50ms

### Monitoring Commands

```bash
# Run tests with timing information
cargo test -- --show-output

# Run specific performance tests
cargo test performance_tests

# Monitor memory usage during tests
cargo test --release -- --show-output
```

### Performance Regression Detection

When tests start failing performance assertions:

1. Check system resources (CPU, memory, disk)
2. Review recent code changes that might affect performance
3. Consider if thresholds need adjustment for the environment
4. Profile specific operations that are slow

## Troubleshooting Guide

### Common Issues and Solutions

#### Database Connection Errors

**Symptom**: Tests fail with "database is locked" or connection errors

**Solutions**:

- Ensure `#[serial]` attribute is used for tests that modify database state
- Check for leftover database files from previous test runs
- Verify temporary file cleanup is working correctly
- Consider increasing connection pool timeout

**Example Fix**:

```rust
#[tokio::test]
#[serial] // Add this for database-modifying tests
async fn test_database_operation() -> Result<()> {
    // test code
}
```

#### Performance Test Failures

**Symptom**: Tests fail with "Operation took longer than expected"

**Solutions**:

- Check system load during test execution
- Review if the operation complexity has increased
- Consider adjusting thresholds for slower environments
- Look for resource contention between concurrent tests

**Example Adjustment**:

```rust
// If tests are consistently slow, consider environment-specific thresholds
let threshold = if cfg!(debug_assertions) {
    Duration::from_millis(200) // More lenient for debug builds
} else {
    performance_thresholds::DATABASE_OPERATION
};
```

#### Flaky Tests

**Symptom**: Tests occasionally fail without code changes

**Solutions**:

- Add `#[serial]` attribute to eliminate race conditions
- Use unique test data suffixes
- Implement retry logic for network-dependent operations
- Increase timeout values for slow operations

**Example Flaky Test Fix**:

```rust
// Before: Flaky due to randomness
let response = server.get("/en/word").await;
assert_eq!(response.json()["word"], "expected_word");

// After: Robust with retry logic
let result = reliability::retry_operation(|| {
    Box::pin(async {
        let response = server.get("/en/word").await;
        // Check response validity instead of specific content
        if response.status_code() == StatusCode::OK {
            Ok(response.json())
        } else {
            Err("Invalid response")
        }
    })
}, 3).await?;
```

#### Memory Usage Issues

**Symptom**: Tests consume excessive memory or fail memory assertions

**Solutions**:

- Check for memory leaks in test setup/teardown
- Ensure database connections are properly closed
- Review bulk operation implementations
- Use memory profilers for detailed analysis

#### Test Data Conflicts

**Symptom**: Tests fail due to data conflicts or constraints

**Solutions**:

- Ensure unique suffixes for all test data
- Use proper cleanup functions between tests
- Check for hardcoded test values that might conflict
- Implement better test isolation

## Adding New Tests

### Checklist for New Tests

- [ ] Follow naming convention: `test_<functionality>_<scenario>`
- [ ] Add appropriate attributes (`#[tokio::test]`, `#[serial]` if needed)
- [ ] Use unique test data suffixes
- [ ] Include both positive and negative test cases
- [ ] Add performance monitoring for critical operations
- [ ] Include proper error handling and cleanup
- [ ] Add descriptive assertions with context messages

### Template for New Test

```rust
#[tokio::test]
#[serial] // Only if modifying database state
async fn test_new_functionality_success() -> Result<()> {
    let (server, _temp_file, pool) = create_test_server_with_pool().await?;

    // Setup test data with unique suffix
    populate_test_data(&pool, "unique_suffix").await?;

    // Perform operation with performance monitoring
    let (response, metrics) = measure_test_performance("operation_name", async {
        Ok(server.get("/api/endpoint").await)
    }).await?;

    // Validate response
    assert_eq!(response.status_code(), StatusCode::OK, "Should return success");

    // Validate performance
    assert_test_performance(&metrics, performance_thresholds::API_REQUEST);

    // Validate response content
    let json: serde_json::Value = response.json();
    assert!(json.get("expected_field").is_some(), "Should have expected field");

    // Cleanup
    cleanup_test_data(&pool).await?;

    Ok(())
}
```

## Dependency Management

### Updating Dependencies

```bash
# Check for outdated dependencies
cargo outdated

# Update dependencies
cargo update

# Test after updates
cargo test
cargo clippy
```

### Critical Dependencies

- `sqlx` - Database operations
- `axum-test` - HTTP testing framework
- `tokio` - Async runtime
- `serial_test` - Test isolation

### Adding New Test Dependencies

When adding new test dependencies:

1. Add to `[dev-dependencies]` section in `Cargo.toml`
2. Document the purpose in this maintenance guide
3. Update test patterns if the dependency changes how tests are written
4. Ensure the dependency is compatible with existing test infrastructure

## Performance Optimization

### Database Performance

- Use connection pooling efficiently
- Minimize database roundtrips in tests
- Use batch operations for bulk test data
- Keep test databases small and focused

### Test Execution Performance

- Use `#[serial]` sparingly - only when necessary
- Parallelize independent tests
- Minimize test setup overhead
- Cache expensive setup operations when possible

### Memory Management

- Clean up large test data structures
- Use appropriate data sizes for tests
- Monitor memory usage in bulk operations
- Avoid memory leaks in long-running test suites

## Test Data Management

### Test Data Strategy

- Use unique suffixes for all test data
- Keep test data minimal but realistic
- Use factories for dynamic test data generation
- Use fixtures for consistent, reusable test data

### Cleanup Strategy

- Automatic cleanup via temporary databases
- Explicit cleanup for shared test data
- Clear documentation of cleanup responsibilities
- Verify cleanup in test teardown

## Debugging Tests

### Debug Mode

```bash
# Run tests with debug output
RUST_LOG=debug cargo test -- --nocapture

# Run a specific test with full output
RUST_LOG=debug cargo test test_specific_function -- --nocapture --exact

# Run specific test file
cargo test --test word_tests -- --nocapture
```

### Common Debug Techniques

- Add temporary `println!` or `dbg!` statements
- Use `--nocapture` flag to see test output
- Enable debug logging for detailed execution traces
- Use debugger breakpoints in IDE for complex issues

### Logging Best Practices

- Use structured logging with context
- Include relevant test data in log messages
- Log performance metrics for analysis
- Remove debug logging before committing

## Quality Assurance

### Code Quality Checks

```bash
# Run all quality checks
cargo test
cargo clippy
cargo fmt --check

# Security audit
cargo audit
```

### Test Quality Metrics

- All tests should have meaningful names
- Each test should focus on a single scenario
- Tests should be independent and isolated
- Performance tests should have appropriate thresholds
- Error messages should be descriptive and actionable

### Review Checklist

Before merging test changes:

- [ ] All tests pass consistently
- [ ] No performance regressions
- [ ] Code follows established patterns
- [ ] Documentation is updated
- [ ] New dependencies are justified
- [ ] Security implications are considered

## Emergency Procedures

### Test Suite Completely Broken

1. Revert to last known good commit
2. Run individual test files to isolate the issue
3. Check for environment or dependency changes
4. Verify database migration compatibility
5. Test with clean database and fresh dependencies

### Performance Degradation

1. Identify which tests are slow using `cargo test -- --show-output`
2. Profile the slow operations
3. Check for resource contention
4. Review recent changes for performance impact
5. Consider temporary threshold adjustments while investigating

### Flaky Test Epidemic

1. Identify patterns in flaky failures
2. Add more `#[serial]` attributes temporarily
3. Implement additional retry logic
4. Increase timeouts for slow environments
5. Review test isolation and cleanup procedures

## Documentation Updates

### When to Update Documentation

- Adding new test patterns
- Changing performance thresholds
- Adding new helper functions
- Modifying test data strategies
- Discovering new troubleshooting solutions

### Documentation Standards

- Keep examples current and working
- Include rationale for design decisions
- Maintain troubleshooting sections based on real issues
- Update performance baselines when infrastructure changes

## Contact and Escalation

For complex test suite issues that can't be resolved using this guide:

1. Review recent commits for breaking changes
2. Check project issue tracker for similar problems
3. Consult with team members familiar with the test infrastructure
4. Consider creating minimal reproduction cases for complex issues

Remember: The goal is maintaining a reliable, fast, and maintainable test suite
that gives confidence in the API's functionality.
