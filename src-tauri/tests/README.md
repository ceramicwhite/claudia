# Claudia Backend Tests

This directory contains comprehensive unit and integration tests for the Claudia Rust backend.

## Test Structure

### Core Test Modules

1. **agents_tests.rs** - Unit tests for agent functionality
   - `AgentRunMetrics::from_jsonl()` - Tests for parsing JSONL output and calculating costs
   - `parse_usage_limit_error()` - Tests for parsing usage limit errors from Claude output
   - `migrate_old_usage_limit_runs()` - Tests for database migration logic
   - Cost calculation tests for different Claude models

2. **scheduler_tests.rs** - Unit tests for the scheduler
   - Scheduler state management tests
   - Finding and updating scheduled runs
   - Status transition tests (scheduled → pending → completed/failed)
   - Concurrent execution safety tests
   - Time-based scheduling logic tests

3. **integration_tests.rs** - Integration tests
   - Agent run creation with usage limits
   - Scheduled run creation and execution flow
   - Metrics calculation and storage
   - Resume functionality and tracking
   - Cost aggregation across multiple runs

4. **edge_cases_tests.rs** - Edge cases and error scenarios
   - Malformed timestamp handling
   - Database constraint violations
   - Extreme token values (MAX, negative, etc.)
   - Invalid JSON structures
   - Timezone edge cases
   - DST transition handling
   - Unknown model pricing fallbacks
   - File system path edge cases
   - Race condition simulations
   - NULL and empty string handling

### Existing Test Modules

- **sandbox/** - Sandbox security tests
- **sandbox_tests.rs** - Main sandbox test runner

## Running Tests

```bash
# Run all tests
cd src-tauri && cargo test

# Run specific test module
cd src-tauri && cargo test agents_tests

# Run with output
cd src-tauri && cargo test -- --nocapture

# Run specific test
cd src-tauri && cargo test test_from_jsonl_with_cost_field
```

## Test Patterns

### Database Tests
Most tests create a temporary SQLite database using `tempfile::TempDir`. This ensures:
- Tests are isolated from each other
- No persistent test data
- Automatic cleanup

### Mock Structures
Some tests use simplified mock structures that mirror the actual implementation to test specific functionality in isolation.

### Time-based Tests
Tests involving scheduling use `chrono` to create deterministic time scenarios, including:
- Past, present, and future scheduling
- Timezone handling
- DST transitions

## Adding New Tests

When adding new tests:
1. Place unit tests in the appropriate module file
2. Add integration tests that span multiple components to `integration_tests.rs`
3. Add edge cases and error scenarios to `edge_cases_tests.rs`
4. Update this README with new test descriptions
5. Follow the existing patterns for database setup and teardown