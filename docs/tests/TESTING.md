# Testing Documentation

## Overview

Claudia's test suite is built with [Vitest](https://vitest.dev/), a modern testing framework that provides fast unit testing with native TypeScript support. The test suite covers React components, services, API integrations, and utility functions with comprehensive mocking of the Tauri API.

## Test Suite Structure

```
src/
├── components/          # Component tests
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
└── test/               # Test utilities and examples
    ├── example.test.tsx
    ├── setup.ts        # Global test setup
    └── tauri-integration.test.tsx
```

## Test Coverage Summary

As of the last test run:
- **Test Files**: 13 (9 passed, 4 failed)
- **Tests**: 412 total (373 passed, 38 failed, 1 skipped)
- **Coverage**: Comprehensive coverage configuration targeting all source files except test files, configs, and Rust code

### Coverage Configuration

```typescript
coverage: {
  provider: 'v8',
  reporter: ['text', 'json', 'html'],
  exclude: [
    'node_modules/',
    'src/test/',
    '**/*.d.ts',
    '**/*.config.*',
    '**/mockdata/**',
    'src-tauri/**',
  ],
}
```

## Running Tests

### Basic Commands

```bash
# Run all tests once
bun run test

# Run tests in watch mode
bun run test:watch

# Run tests with UI interface
bun run test:ui

# Run tests with coverage report
bun run test:coverage

# Run specific test file
bun test src/lib/api.test.ts

# Run tests matching a pattern
bun test --grep "api"
```

### Environment Setup

Tests run in a jsdom environment with comprehensive Tauri API mocking. The setup file (`src/test/setup.ts`) provides:

1. **Tauri API Mocking**: All Tauri APIs are mocked to enable testing without the desktop runtime
2. **Window API Polyfills**: Browser APIs like `matchMedia`, `IntersectionObserver`, and `ResizeObserver`
3. **Testing Library Integration**: Jest DOM matchers for improved assertions
4. **Timer Function Polyfills**: Support for fake timers in Bun environment

## Key Test Scenarios

### 1. Component Testing

Component tests use React Testing Library for user-centric testing:

```typescript
// Example from button.test.tsx
describe('Button', () => {
  it('handles click events', async () => {
    const user = userEvent.setup()
    const handleClick = vi.fn()
    render(<Button onClick={handleClick}>Click me</Button>)
    
    const button = screen.getByRole('button')
    await user.click(button)
    
    expect(handleClick).toHaveBeenCalledTimes(1)
  })
})
```

### 2. Service Layer Testing

Service tests focus on the BaseService pattern and error handling:

```typescript
// Example from base.service.test.ts
describe('BaseService', () => {
  it('should retry on retryable errors', async () => {
    const networkError = new AppError(ErrorCode.NETWORK_ERROR, 'Network failed');
    
    mockInvoke
      .mockRejectedValueOnce(networkError)
      .mockRejectedValueOnce(networkError)
      .mockResolvedValueOnce('success');

    const result = await service.testInvoke('retry_command', {}, retrySchema);
    expect(result).toBe('success');
    expect(mockInvoke).toHaveBeenCalledTimes(3);
  })
})
```

### 3. API Integration Testing

API tests ensure proper command invocation and error handling:

```typescript
// Example from api.test.ts
describe('api', () => {
  it('should create a scheduled agent run', async () => {
    const expectedRunId = 123
    mockInvoke.mockResolvedValueOnce(expectedRunId)

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
    expect(result).toBe(expectedRunId)
  })
})
```

### 4. Error Handling Testing

Comprehensive error testing with custom error classes:

```typescript
// Example from errors.test.ts
describe('AppError', () => {
  it('should serialize to JSON correctly', () => {
    const error = new AppError(
      ErrorCode.VALIDATION,
      'Validation failed',
      { field: 'email' }
    );

    const json = error.toJSON();
    expect(json).toEqual({
      code: 'VALIDATION',
      message: 'Validation failed',
      details: { field: 'email' },
      timestamp: expect.any(String),
      stackTrace: expect.any(String)
    });
  });
});
```

## Testing Patterns

### 1. Mock Setup Pattern

All tests follow a consistent mock setup pattern:

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
  // tests...
})
```

### 2. Async Testing Pattern

Proper handling of async operations:

```typescript
it('should handle async operations', async () => {
  // Setup
  mockInvoke.mockResolvedValueOnce(expectedValue)
  
  // Execute
  const result = await api.someAsyncMethod()
  
  // Assert
  expect(result).toBe(expectedValue)
})
```

### 3. Error Testing Pattern

Consistent error testing approach:

```typescript
it('should handle errors correctly', async () => {
  const error = new Error('Test error')
  mockInvoke.mockRejectedValueOnce(error)

  await expect(api.someMethod()).rejects.toThrow('Test error')
})
```

### 4. Component Event Testing

User interaction testing with Testing Library:

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

## Mock Utilities

### Tauri API Mocks

The setup file provides comprehensive Tauri mocking:

```typescript
// Window internals
window.__TAURI_INTERNALS__ = {
  invoke: vi.fn(() => Promise.resolve()),
}

// API modules
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(() => Promise.resolve()),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
  emit: vi.fn(),
  once: vi.fn(),
}))
```

### Browser API Mocks

Essential browser APIs for component testing:

```typescript
// IntersectionObserver
global.IntersectionObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}))

// ResizeObserver
global.ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}))
```

## Known Limitations

### Current Test Failures

1. **Type Export Tests**: Some type definition tests are failing due to module resolution issues
2. **Component Event Tests**: Certain event handling tests have timing issues that need addressing
3. **Async Assertions**: Some tests have unhandled promise rejections that need proper await handling

### Testing Constraints

1. **Tauri Runtime**: Tests cannot access actual Tauri runtime features
2. **File System**: File operations are mocked and don't touch the real filesystem
3. **Database**: SQLite operations are not tested directly in the frontend tests
4. **WebView Features**: Platform-specific WebView features cannot be tested

## CI/CD Integration

### GitHub Actions Setup

Add this workflow to `.github/workflows/test.yml`:

```yaml
name: Test

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - uses: oven-sh/setup-bun@v1
        with:
          bun-version: latest
          
      - name: Install dependencies
        run: bun install
        
      - name: Run tests
        run: bun run test:coverage
        
      - name: Upload coverage
        uses: actions/upload-artifact@v3
        with:
          name: coverage-report
          path: coverage/
```

### Pre-commit Hooks

Add testing to your pre-commit workflow:

```bash
#!/bin/sh
# .git/hooks/pre-commit

# Run tests
bun run test:run

# Check the exit code
if [ $? -ne 0 ]; then
  echo "Tests failed. Please fix before committing."
  exit 1
fi
```

## Adding New Tests

### Component Test Template

```typescript
import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MyComponent } from './MyComponent'

describe('MyComponent', () => {
  it('should render correctly', () => {
    render(<MyComponent />)
    expect(screen.getByText('Expected Text')).toBeInTheDocument()
  })

  it('should handle user interaction', async () => {
    const user = userEvent.setup()
    const onAction = vi.fn()
    
    render(<MyComponent onAction={onAction} />)
    
    await user.click(screen.getByRole('button'))
    
    expect(onAction).toHaveBeenCalled()
  })
})
```

### Service Test Template

```typescript
import { describe, it, expect, vi, beforeEach } from 'vitest'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: any[]) => mockInvoke(...args),
}))

import { MyService } from './MyService'

describe('MyService', () => {
  let service: MyService
  
  beforeEach(() => {
    vi.clearAllMocks()
    service = new MyService()
  })

  it('should perform operation', async () => {
    mockInvoke.mockResolvedValueOnce({ success: true })
    
    const result = await service.doOperation()
    
    expect(mockInvoke).toHaveBeenCalledWith('my_command', expect.any(Object))
    expect(result).toEqual({ success: true })
  })
})
```

## Common Pitfalls

### 1. Mock Order
Always mock modules before importing them:
```typescript
// ❌ Wrong
import { api } from './api'
vi.mock('@tauri-apps/api/core')

// ✅ Correct
vi.mock('@tauri-apps/api/core')
import { api } from './api'
```

### 2. Async Assertions
Always await async expectations:
```typescript
// ❌ Wrong
expect(asyncFunction()).rejects.toThrow()

// ✅ Correct
await expect(asyncFunction()).rejects.toThrow()
```

### 3. Cleanup
Always clean up after tests:
```typescript
beforeEach(() => {
  vi.clearAllMocks()
})

afterEach(() => {
  cleanup() // for React components
})
```

### 4. Timer Testing
Use fake timers for time-dependent tests:
```typescript
beforeEach(() => {
  vi.useFakeTimers()
})

afterEach(() => {
  vi.useRealTimers()
})

it('should handle timeouts', async () => {
  const promise = functionWithTimeout()
  
  await vi.runAllTimersAsync()
  
  await expect(promise).resolves.toBe('done')
})
```

## Future Improvements

### Planned Enhancements

1. **E2E Testing**: Implement Playwright tests for full application flows
2. **Visual Regression**: Add visual regression testing for UI components
3. **Performance Testing**: Implement performance benchmarks for critical paths
4. **Mutation Testing**: Add mutation testing to verify test quality
5. **Test Data Factories**: Create factories for consistent test data generation

### Coverage Goals

- Target: 80%+ code coverage for all new code
- Critical paths: 90%+ coverage for services and API layers
- UI Components: Focus on behavior over implementation details

### Testing Infrastructure

1. **Parallel Test Execution**: Enable parallel test runs for faster CI
2. **Test Sharding**: Split tests across multiple runners
3. **Flaky Test Detection**: Implement retry logic for flaky tests
4. **Test Result History**: Track test performance over time

## Resources

- [Vitest Documentation](https://vitest.dev/)
- [React Testing Library](https://testing-library.com/docs/react-testing-library/intro/)
- [Testing Best Practices](https://github.com/goldbergyoni/javascript-testing-best-practices)
- [Tauri Testing Guide](https://tauri.app/v1/guides/testing/)