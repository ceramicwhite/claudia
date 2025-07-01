# Testing Documentation

This section covers testing strategies, patterns, and guidelines for Claudia.

## Overview

The Claudia testing suite provides comprehensive coverage across both frontend and backend components. For a complete overview of test results and metrics, see the [Comprehensive Test Report](./COMPREHENSIVE_TEST_REPORT.md).

## Key Documents

### Reports and Plans
- [Comprehensive Test Report](./COMPREHENSIVE_TEST_REPORT.md) - Complete test coverage analysis and results
- [Test Plan](./TEST_PLAN.md) - Overall testing strategy and roadmap
- [Agent Test Plan](./AGENT_TEST_PLAN.md) - Specialized testing for agent functionality
- [Testing Guidelines](./TESTING.md) - Best practices and patterns

### Implementation Guides
- [Frontend Tests Guide](./frontend-tests-readme.md) - React component and service testing
- [Rust Tests Guide](./rust-tests-readme.md) - Backend unit and integration testing
- [Sandbox Tests Guide](./sandbox-tests-readme.md) - Security sandbox testing

## Current Test Coverage

- **Overall Coverage**: 82% (87% backend, 76% frontend)
- **Total Tests**: 342 (127 frontend, 215 backend)
- **Execution Time**: ~3.5 minutes (full suite)

For detailed metrics, see the [Comprehensive Test Report](./COMPREHENSIVE_TEST_REPORT.md).

## Testing Stack

### Frontend
- **Test Runner**: Vitest
- **Testing Library**: React Testing Library
- **Assertions**: Vitest built-in matchers
- **Coverage**: Vitest coverage

### Backend
- **Test Framework**: Rust built-in testing
- **Assertions**: Standard Rust assertions
- **Integration**: Custom test harness

## Quick Commands

```bash
# Frontend tests
bun run test         # Run in watch mode
bun run test:run     # Run once
bun run test:ui      # With UI
bun run test:coverage # Coverage report

# Backend tests
cd src-tauri && cargo test              # All tests
cd src-tauri && cargo test -- --test-threads=1  # Sequential
cd src-tauri && cargo test test_name    # Specific test
```

## Best Practices

1. **Write tests first**: Follow TDD when possible
2. **Test behavior, not implementation**: Focus on what, not how
3. **Keep tests simple**: One assertion per test when practical
4. **Use descriptive names**: Test names should explain what they test
5. **Maintain test data**: Keep fixtures up to date

## Test Organization

```
src/
├── components/
│   ├── Component.tsx
│   └── Component.test.tsx
├── services/
│   ├── service.ts
│   └── service.test.ts
└── test/
    ├── setup.ts
    └── utils.tsx

src-tauri/tests/
├── integration_tests.rs
├── unit_tests/
└── common/
    └── mod.rs
```