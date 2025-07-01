# Claudia Testing Guide

This document consolidates all testing information for the Claudia project, covering both frontend (Vitest) and backend (Rust) testing approaches.

## Table of Contents

1. [Overview](#overview)
2. [Frontend Testing](#frontend-testing)
3. [Backend Testing](#backend-testing)
4. [Running Tests](#running-tests)
5. [Test Organization](#test-organization)
6. [Testing Patterns](#testing-patterns)
7. [Coverage Targets](#coverage-targets)
8. [CI/CD Integration](#cicd-integration)
9. [Common Issues and Solutions](#common-issues-and-solutions)

## Overview

Claudia employs a comprehensive testing strategy with different approaches for frontend and backend:

- **Frontend**: Vitest + React Testing Library for component and integration tests
- **Backend**: Rust's built-in test framework with cargo test
- **Coverage Goals**: 80%+ overall, 90%+ for critical paths, 100% for security-critical code

## Frontend Testing

### Technology Stack

- **Framework**: [Vitest](https://vitest.dev/) - Fast unit testing with native TypeScript support
- **Component Testing**: React Testing Library - User-centric testing approach
- **User Interactions**: @testing-library/user-event - Realistic user event simulation
- **DOM Environment**: jsdom - Browser DOM implementation for Node.js
- **Coverage**: @vitest/coverage-v8 - Code coverage reporting

### Frontend Test Structure

```
src/
├── components/          # Component tests (*.test.tsx)
│   ├── AgentExecution.test.tsx
│   ├── RunningSessionsView.test.tsx
│   ├── SessionCard.test.tsx
│   ├── ToolWidgets.test.tsx
│   └── ui/
│       ├── button.test.tsx
│       └── date-time-picker.test.tsx
├── lib/                 # Library tests
│   ├── api.test.ts
│   ├── api.comprehensive.test.ts
│   └── errors.test.ts
├── services/           # Service layer tests
│   ├── base.service.test.ts
│   └── project.service.test.ts
└── test/               # Test utilities
    ├── setup.ts        # Global test setup
    └── utils.tsx       # Test helpers
```

### Key Frontend Test Scenarios

#### 1. Component Testing
Tests focus on user interactions and behavior:

```typescript
describe('Button', () => {
  it('handles click events', async () => {
    const user = userEvent.setup()
    const handleClick = vi.fn()
    render(<Button onClick={handleClick}>Click me</Button>)
    
    await user.click(screen.getByRole('button'))
    expect(handleClick).toHaveBeenCalledTimes(1)
  })
})
```

#### 2. Service Layer Testing
Tests validate API calls and error handling:

```typescript
describe('BaseService', () => {
  it('should retry on retryable errors', async () => {
    const networkError = new AppError(ErrorCode.NETWORK_ERROR, 'Network failed')
    
    mockInvoke
      .mockRejectedValueOnce(networkError)
      .mockRejectedValueOnce(networkError)
      .mockResolvedValueOnce('success')

    const result = await service.testInvoke('retry_command', {}, retrySchema)
    expect(result).toBe('success')
    expect(mockInvoke).toHaveBeenCalledTimes(3)
  })
})
```

#### 3. API Integration Testing
Tests ensure proper Tauri command invocation:

```typescript
describe('api', () => {
  it('should create a scheduled agent run', async () => {
    mockInvoke.mockResolvedValueOnce(123)

    const result = await api.createScheduledAgentRun(
      1, '/test/project', 'Test task', 'claude-3-sonnet', '2024-01-15T10:30:00Z'
    )

    expect(mockInvoke).toHaveBeenCalledWith('create_scheduled_agent_run', {
      agentId: 1,
      projectPath: '/test/project',
      task: 'Test task',
      model: 'claude-3-sonnet',
      scheduledStartTime: '2024-01-15T10:30:00Z'
    })
  })
})
```

### Frontend Testing Patterns

#### Mock Setup Pattern
```typescript
// Mock before imports
const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: any[]) => mockInvoke(...args),
}))

// Import after mocking
import { api } from './api'

describe('api', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })
})
```

#### Async Testing Pattern
```typescript
it('should handle async operations', async () => {
  mockInvoke.mockResolvedValueOnce(expectedValue)
  const result = await api.someAsyncMethod()
  expect(result).toBe(expectedValue)
})
```

#### Component Event Testing
```typescript
it('should handle user interactions', async () => {
  const user = userEvent.setup()
  const onSubmit = vi.fn()
  
  render(<Form onSubmit={onSubmit} />)
  
  await user.type(screen.getByLabelText('Name'), 'Test User')
  await user.click(screen.getByRole('button', { name: 'Submit' }))
  
  expect(onSubmit).toHaveBeenCalledWith({ name: 'Test User' })
})
```

## Backend Testing

### Rust Test Structure

```
src-tauri/
├── tests/              # Integration tests
│   ├── agents_tests.rs
│   ├── scheduler_tests.rs
│   ├── integration_tests.rs
│   ├── edge_cases_tests.rs
│   └── sandbox/
└── src/
    └── commands/
        └── agents/     # Unit tests in module files
            ├── types.rs (with #[cfg(test)] mod tests)
            ├── repository.rs
            └── service.rs
```

### Backend Test Categories

#### 1. Unit Tests
Located within module files using `#[cfg(test)]`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_id_validation() {
        assert!(AgentId::new(1).is_ok());
        assert!(AgentId::new(0).is_err());
        assert!(AgentId::new(-1).is_err());
    }
}
```

#### 2. Integration Tests
Test complete workflows across modules:

```rust
#[tokio::test]
async fn test_agent_execution_workflow() {
    let pool = setup_test_db();
    let service = AgentService::new(app_handle);
    
    // Create agent
    let agent = service.create_agent(...).await.unwrap();
    
    // Execute agent
    let run = service.execute_agent(...).await.unwrap();
    
    // Verify execution
    assert_eq!(run.status, "running");
}
```

#### 3. Edge Case Tests
Handle unusual scenarios and error conditions:

```rust
#[test]
fn test_malformed_timestamp_handling() {
    let invalid_timestamps = vec![
        "2024-13-01T00:00:00Z",  // Invalid month
        "2024-01-32T00:00:00Z",  // Invalid day
        "not-a-timestamp",        // Completely invalid
    ];
    
    for ts in invalid_timestamps {
        assert!(parse_datetime(ts).is_err());
    }
}
```

### Key Backend Test Areas

#### Agent Module Tests
- Type validation (AgentId, RunId, SessionId)
- Database CRUD operations
- Process lifecycle management
- Metrics calculation
- Error handling and recovery

#### Database Tests
- Schema migrations
- Connection pooling
- Transaction handling
- Concurrent access
- Foreign key constraints

#### Security Tests
- Sandbox rule generation
- Platform-specific sandboxing
- Permission validation
- Violation tracking

## Running Tests

### Frontend Tests

```bash
# Run all frontend tests in watch mode
bun run test

# Run tests once
bun run test:run

# Run with UI
bun run test:ui

# Run with coverage
bun run test:coverage

# Run specific test file
bun test src/lib/api.test.ts

# Run tests matching pattern
bun test --grep "api"
```

### Backend Tests

```bash
# Run all backend tests
cd src-tauri && cargo test

# Run specific test module
cd src-tauri && cargo test agents_tests

# Run with output
cd src-tauri && cargo test -- --nocapture

# Run specific test
cd src-tauri && cargo test test_from_jsonl_with_cost_field

# Run tests for a specific module
cd src-tauri && cargo test --package claudia --lib commands::agents
```

## Test Organization

### Test Priorities

#### Priority 1: Critical (Core Functionality) - Target: 90%+
- API Layer
- Base Service
- Error Handling
- Database Operations
- Process Management

#### Priority 2: High (Service Layer) - Target: 85%+
- Agent Service
- Session Service
- Claude Service
- Project Service

#### Priority 3: Medium (UI Components) - Target: 80%+
- RunningSessionsView
- SessionCard
- AgentExecution
- SessionList
- ToolWidgets

#### Priority 4: Low (Utilities) - Target: 75%+
- Hooks
- Error Handler Utility
- Schemas
- Widget Components

## Testing Patterns

### Database Testing Pattern
```rust
fn setup_test_db() -> SqlitePool {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = create_pool(db_path).unwrap();
    init_pool_db(&pool).unwrap();
    pool
}

#[test]
fn test_database_operation() {
    let pool = setup_test_db();
    // Test operations
    // Automatic cleanup when pool goes out of scope
}
```

### Mock Service Pattern
```rust
#[tokio::test]
async fn test_service_operation() {
    let app = mock_builder().build(tauri::generate_context!()).unwrap();
    let service = MyService::new(app.handle());
    
    // Test service methods
}
```

### Error Testing Pattern
```typescript
it('should handle errors correctly', async () => {
  const error = new Error('Test error')
  mockInvoke.mockRejectedValueOnce(error)

  await expect(api.someMethod()).rejects.toThrow('Test error')
})
```

## Coverage Targets

| Module Type | Target Coverage | Rationale |
|------------|----------------|-----------|
| Critical Services | 90%+ | Core functionality, high impact |
| Service Layer | 85%+ | Business logic, medium impact |
| UI Components | 80%+ | User-facing, visual testing important |
| Utilities | 75%+ | Helper functions, lower risk |
| Type Definitions | 70%+ | Compile-time safety primary |

## CI/CD Integration

### GitHub Actions Configuration

```yaml
name: Test Suite

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  frontend-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: oven-sh/setup-bun@v1
      - run: bun install
      - run: bun run test:coverage
      - uses: codecov/codecov-action@v3

  backend-tests:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cd src-tauri && cargo test
```

### Pre-commit Hooks

```bash
#!/bin/sh
# .git/hooks/pre-commit

# Run frontend tests
bun run test:run

# Run backend tests
cd src-tauri && cargo test

# Check formatting
cd src-tauri && cargo fmt -- --check
```

## Common Issues and Solutions

### Frontend Testing Issues

#### 1. Mock Order Problems
```typescript
// ❌ Wrong
import { api } from './api'
vi.mock('@tauri-apps/api/core')

// ✅ Correct
vi.mock('@tauri-apps/api/core')
import { api } from './api'
```

#### 2. Async Assertion Issues
```typescript
// ❌ Wrong
expect(asyncFunction()).rejects.toThrow()

// ✅ Correct
await expect(asyncFunction()).rejects.toThrow()
```

#### 3. Cleanup Issues
```typescript
beforeEach(() => {
  vi.clearAllMocks()
})

afterEach(() => {
  cleanup() // for React components
})
```

### Backend Testing Issues

#### 1. Database Lock Issues
- Use separate test databases
- Ensure proper connection cleanup
- Avoid concurrent database modifications

#### 2. Async Test Issues
```rust
// Use #[tokio::test] for async tests
#[tokio::test]
async fn test_async_operation() {
    // async test code
}
```

#### 3. Platform-specific Tests
```rust
#[cfg(target_os = "macos")]
#[test]
fn test_macos_specific() {
    // macOS-specific test
}
```

## Test Data Management

### Frontend Fixtures
```typescript
// test/fixtures/agents.ts
export const mockAgent = {
  id: 1,
  name: "Test Agent",
  icon: "🤖",
  model: "sonnet",
  // ... other fields
}
```

### Backend Fixtures
```rust
// tests/fixtures/mod.rs
pub fn create_test_agent() -> NewAgent {
    NewAgent {
        name: "Test Agent".to_string(),
        icon: "🤖".to_string(),
        system_prompt: "Test prompt".to_string(),
        // ... other fields
    }
}
```

## Future Testing Improvements

### Planned Enhancements
1. **E2E Testing**: Playwright tests for full application flows
2. **Visual Regression**: Screenshot comparison testing
3. **Performance Testing**: Benchmarks for critical paths
4. **Mutation Testing**: Verify test quality
5. **Test Data Factories**: Consistent test data generation

### Testing Infrastructure
1. **Parallel Test Execution**: Speed up CI runs
2. **Test Sharding**: Distribute tests across runners
3. **Flaky Test Detection**: Automatic retry logic
4. **Test Result History**: Track performance over time

## Resources

- [Vitest Documentation](https://vitest.dev/)
- [React Testing Library](https://testing-library.com/docs/react-testing-library/intro/)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tauri Testing Guide](https://tauri.app/v1/guides/testing/)
- [Testing Best Practices](https://github.com/goldbergyoni/javascript-testing-best-practices)