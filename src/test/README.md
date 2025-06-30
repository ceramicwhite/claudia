# Frontend Testing Guide

This directory contains the test setup and utilities for testing the Claudia frontend application.

## Test Stack

- **Vitest**: Fast unit test framework with Vite integration
- **React Testing Library**: Testing utilities for React components
- **@testing-library/user-event**: Simulating user interactions
- **jsdom**: DOM implementation for Node.js
- **@vitest/coverage-v8**: Code coverage reporting

## Running Tests

```bash
# Run tests in watch mode
bun test

# Run tests once
bun test:run

# Run tests with UI
bun test:ui

# Run tests with coverage
bun test:coverage
```

## Test Structure

- `setup.ts`: Global test setup including Tauri API mocks
- `utils.tsx`: Common testing utilities and custom render functions
- `*.test.tsx`: Test files colocated with components or in this directory

## Writing Tests

### Basic Component Test

```tsx
import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { MyComponent } from '../components/MyComponent'

describe('MyComponent', () => {
  it('renders correctly', () => {
    render(<MyComponent />)
    expect(screen.getByText('Hello')).toBeInTheDocument()
  })
})
```

### Testing with Tauri API

```tsx
import { vi } from 'vitest'
import { invoke } from '@tauri-apps/api'

it('calls Tauri command', async () => {
  vi.mocked(invoke).mockResolvedValueOnce({ data: 'test' })
  
  // Your test code
  const result = await invoke('my_command')
  
  expect(invoke).toHaveBeenCalledWith('my_command')
  expect(result).toEqual({ data: 'test' })
})
```

### Testing User Interactions

```tsx
import userEvent from '@testing-library/user-event'

it('handles user input', async () => {
  const user = userEvent.setup()
  render(<MyForm />)
  
  const input = screen.getByRole('textbox')
  await user.type(input, 'Hello world')
  
  expect(input).toHaveValue('Hello world')
})
```

## Mocked APIs

The following Tauri APIs are automatically mocked:

- `@tauri-apps/api` - Core API functions (invoke, convertFileSrc)
- `@tauri-apps/api/event` - Event system (listen, emit, once)
- `@tauri-apps/api/window` - Window management
- `@tauri-apps/api/path` - Path utilities
- `@tauri-apps/plugin-dialog` - Dialog APIs
- `@tauri-apps/plugin-shell` - Shell commands
- `@tauri-apps/plugin-global-shortcut` - Keyboard shortcuts
- `@tauri-apps/plugin-opener` - External opener

## Best Practices

1. **Use data-testid sparingly**: Prefer accessible queries (getByRole, getByLabelText, etc.)
2. **Test behavior, not implementation**: Focus on what users see and do
3. **Keep tests isolated**: Each test should be independent
4. **Mock external dependencies**: Especially Tauri APIs and network requests
5. **Use descriptive test names**: Make it clear what is being tested
6. **Avoid testing implementation details**: Don't test state variables directly

## Coverage

Coverage reports are generated in the `coverage/` directory. View the HTML report:

```bash
bun test:coverage
open coverage/index.html
```

## Debugging Tests

- Use `test.only()` to run a single test
- Use `test.skip()` to temporarily skip tests
- Add `console.log()` statements or use the debugger
- Run tests with UI for interactive debugging: `bun test:ui`