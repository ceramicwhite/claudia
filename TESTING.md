# Testing Documentation

This document provides comprehensive information about the test suite structure, coverage metrics, and testing best practices for the Claudia project.

## Overview

Claudia uses a multi-layered testing approach with separate test suites for the frontend (TypeScript/React) and backend (Rust/Tauri) components.

### Testing Stack
- **Frontend**: Vitest + React Testing Library + Jest DOM
- **Backend**: Rust's built-in test framework + cargo test
- **Coverage**: Vitest Coverage (v8) for frontend, cargo-tarpaulin (optional) for Rust

## Test Suite Structure

### Frontend Tests (`/src`)
```
src/
├── test/
│   ├── setup.ts              # Test environment setup and global mocks
│   ├── example.test.tsx      # Example test patterns
│   └── tauri-integration.test.tsx  # Tauri API integration tests
├── components/
│   ├── SessionCard.test.tsx  # Component unit tests
│   ├── RunningSessionsView.test.tsx
│   └── ui/
│       ├── button.test.tsx
│       └── date-time-picker.test.tsx
└── lib/
    └── api.test.ts           # API wrapper tests
```

### Backend Tests (`/src-tauri/tests`)
```
src-tauri/tests/
├── agents_tests.rs          # Agent execution tests
├── edge_cases_tests.rs      # Edge case handling
├── integration_tests.rs     # Integration scenarios
├── scheduler_tests.rs       # Scheduler functionality
└── sandbox_tests.rs         # Security sandbox tests
```

## Running Tests

### Frontend Tests

```bash
# Run all tests
bun run test

# Run tests with UI
bun run test:ui

# Run tests once (no watch)
bun run test:run

# Generate coverage report
bun run test:coverage
```

### Backend Tests

```bash
# Run all Rust tests
cd src-tauri && cargo test

# Run specific test file
cd src-tauri && cargo test --test agents_tests

# Run with verbose output
cd src-tauri && cargo test -- --nocapture

# Run library tests only
cd src-tauri && cargo test --lib
```

## Test Coverage Metrics

### Frontend Coverage (as of latest run)

**Overall Coverage**: 6.84%

| Category | Statements | Branches | Functions | Lines |
|----------|------------|----------|-----------|-------|
| All files | 6.84% | 68.42% | 26.48% | 6.84% |
| Components | 4.12% | 72.32% | 42.64% | 4.12% |
| UI Components | 33.28% | 65.88% | 34.14% | 33.28% |
| Library | 13.61% | 75% | 14.85% | 13.61% |

**Well-tested components:**
- `SessionCard.tsx`: 99.11% coverage
- `RunningSessionsView.tsx`: 96.49% coverage
- `button.tsx`: 100% coverage
- `badge.tsx`: 100% coverage
- `utils.ts`: 100% coverage

**Areas needing improvement:**
- Main application components (App.tsx, Settings.tsx)
- Complex UI components (ClaudeSession.tsx, AgentExecution.tsx)
- API integration layers

### Backend Coverage

Currently, the backend has:
- 1 library test passing
- Multiple integration test files with compilation issues due to missing dev dependencies
- Test infrastructure in place but needs dependency updates

## Key Test Scenarios

### Frontend Testing Patterns

1. **Component Testing**
   ```typescript
   // Example from SessionCard.test.tsx
   it('should display session metrics correctly', () => {
     render(<SessionCard session={mockSession} />);
     expect(screen.getByText('1,234 tokens')).toBeInTheDocument();
     expect(screen.getByText('$0.12')).toBeInTheDocument();
   });
   ```

2. **Tauri API Mocking**
   ```typescript
   // Mocking Tauri invoke calls
   vi.mocked(invoke).mockResolvedValueOnce({ data: 'test' });
   const result = await invoke('test_command', { arg: 'value' });
   ```

3. **Event Testing**
   ```typescript
   // Testing user interactions
   const user = userEvent.setup();
   await user.click(screen.getByRole('button'));
   expect(mockCallback).toHaveBeenCalled();
   ```

### Backend Testing Patterns

1. **Unit Tests**
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_checkpoint_state_lifecycle() {
           // Test implementation
       }
   }
   ```

2. **Integration Tests**
   ```rust
   #[tokio::test]
   async fn test_agent_execution() {
       // Test async operations
   }
   ```

## Testing Best Practices

### General Guidelines

1. **Test Naming**: Use descriptive names that explain what is being tested
   ```typescript
   // Good
   it('should show error toast when session fails to load')
   
   // Bad
   it('test error')
   ```

2. **Test Organization**: Group related tests using `describe` blocks
3. **Mock External Dependencies**: Always mock Tauri APIs, network calls, and file system operations
4. **Test User Behavior**: Focus on testing what users see and do, not implementation details

### Frontend Specific

1. **Use React Testing Library queries in order of preference**:
   - `getByRole` > `getByLabelText` > `getByText` > `getByTestId`

2. **Wait for async operations**:
   ```typescript
   await waitFor(() => {
     expect(screen.getByText('Loaded')).toBeInTheDocument();
   });
   ```

3. **Test accessibility**: Ensure components are keyboard navigable and screen reader friendly

### Backend Specific

1. **Use `#[cfg(test)]` for test modules**: Keeps test code out of production builds
2. **Test both success and error paths**: Ensure proper error handling
3. **Use fixtures for complex test data**: Maintain consistency across tests
4. **Test concurrent operations**: Verify thread safety and async behavior

## CI/CD Integration

### GitHub Actions Configuration

Create `.github/workflows/test.yml`:

```yaml
name: Test

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test-frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: oven-sh/setup-bun@v1
      - run: bun install
      - run: bun run test:coverage
      - uses: codecov/codecov-action@v3
        with:
          files: ./coverage/coverage-final.json

  test-backend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cd src-tauri && cargo test --lib
```

## Known Limitations

### Current Issues

1. **Backend test dependencies**: Several test files require additional dev dependencies:
   - `once_cell`
   - `parking_lot`
   - `pretty_assertions`
   - `test_case`
   - `serial_test`

2. **Coverage gaps**: Many core components lack test coverage
3. **Integration test setup**: Full Tauri integration tests require additional setup
4. **Platform-specific tests**: Sandbox tests may behave differently across OS

### Temporary Workarounds

- Run `cargo test --lib` for backend to avoid integration test failures
- Focus on unit tests until integration test dependencies are resolved
- Use manual testing for platform-specific features

## Future Testing Improvements

### High Priority

1. **Increase test coverage**:
   - Target 80% coverage for critical paths
   - Add tests for all API endpoints
   - Test error boundaries and edge cases

2. **Fix backend test dependencies**:
   ```toml
   # Add to Cargo.toml [dev-dependencies]
   once_cell = "1.19"
   parking_lot = "0.12"
   pretty_assertions = "1.4"
   test_case = "3.3"
   serial_test = "3.0"
   ```

3. **Add E2E tests**: Use Playwright or Cypress for full application testing

### Medium Priority

1. **Performance testing**: Add benchmarks for critical operations
2. **Visual regression testing**: Ensure UI consistency
3. **Mutation testing**: Verify test quality
4. **Property-based testing**: Use proptest for Rust components

### Low Priority

1. **Snapshot testing**: For complex UI components
2. **Fuzz testing**: For security-critical inputs
3. **Contract testing**: For API boundaries

## Writing New Tests

### Frontend Test Template

```typescript
import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { YourComponent } from './YourComponent'

describe('YourComponent', () => {
  it('should render correctly', () => {
    render(<YourComponent />);
    expect(screen.getByRole('button')).toBeInTheDocument();
  });

  it('should handle user interaction', async () => {
    const user = userEvent.setup();
    const onClick = vi.fn();
    
    render(<YourComponent onClick={onClick} />);
    await user.click(screen.getByRole('button'));
    
    expect(onClick).toHaveBeenCalled();
  });
});
```

### Backend Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_your_function() {
        // Arrange
        let input = "test";
        
        // Act
        let result = your_function(input);
        
        // Assert
        assert_eq!(result, expected_value);
    }

    #[tokio::test]
    async fn test_async_function() {
        // Test async operations
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

## Debugging Tests

### Frontend
- Use `screen.debug()` to see the current DOM
- Set `DEBUG_PRINT_LIMIT=0` for unlimited debug output
- Use VS Code's Vitest extension for debugging

### Backend
- Use `cargo test -- --nocapture` to see println! output
- Add `RUST_LOG=debug` for detailed logging
- Use `cargo test -- --test-threads=1` for sequential execution

## Resources

- [Vitest Documentation](https://vitest.dev/)
- [React Testing Library](https://testing-library.com/docs/react-testing-library/intro/)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tauri Testing Guide](https://tauri.app/v1/guides/testing/)
- [Testing Best Practices](https://kentcdodds.com/blog/common-mistakes-with-react-testing-library)

---

*Last updated: December 2024*