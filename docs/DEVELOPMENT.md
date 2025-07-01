# Claudia Development Guide

This guide covers everything you need to know to develop and contribute to Claudia.

## Development Setup

### Prerequisites

Ensure you have all the prerequisites from the [Installation Guide](INSTALLATION.md#build-prerequisites) installed:
- Rust (1.70.0+)
- Bun (latest)
- Git
- Platform-specific dependencies

### Getting Started

1. **Fork and Clone**
   ```bash
   # Fork the repository on GitHub first
   git clone https://github.com/YOUR_USERNAME/claudia.git
   cd claudia
   ```

2. **Install Dependencies**
   ```bash
   # Install frontend dependencies
   bun install
   
   # Build Rust dependencies
   cd src-tauri
   cargo build
   cd ..
   ```

3. **Start Development Server**
   ```bash
   # Run the full Tauri application
   bun run tauri dev
   
   # Or run frontend only (faster for UI development)
   bun run dev
   ```

## Development Commands

### Frontend Development

```bash
# Start Vite dev server (frontend only)
bun run dev

# Type checking
bunx tsc --noEmit

# Run tests
bun run test              # Watch mode
bun run test:ui           # With UI
bun run test:run          # Run once
bun run test:coverage     # Coverage report

# Build frontend only
bun run build

# Preview production build
bun run preview
```

### Backend Development

```bash
# Change to Rust directory
cd src-tauri

# Format code
cargo fmt

# Check code
cargo clippy

# Run tests
cargo test              # All tests
cargo test --lib        # Unit tests only
cargo test --doc        # Documentation tests

# Build
cargo build             # Debug build
cargo build --release   # Release build
```

### Full Application

```bash
# Development mode with hot reload
bun run tauri dev

# Production build
bun run tauri build

# Build with specific features
bun run tauri build --features "feature1,feature2"
```

## Project Structure

### Frontend (`/src`)

```
src/
├── components/           # React components
│   ├── ui/              # Reusable UI components
│   ├── agents/          # Agent-related components
│   ├── projects/        # Project management
│   ├── usage/           # Usage analytics
│   └── ...              # Feature-specific components
├── services/            # API service layer
├── lib/                 # Utilities and helpers
│   ├── api.ts          # Tauri command wrapper
│   ├── utils.ts        # Common utilities
│   └── types.ts        # TypeScript types
├── hooks/               # Custom React hooks
├── contexts/            # React contexts
├── assets/              # Static assets
└── App.tsx              # Main application
```

### Backend (`/src-tauri`)

```
src-tauri/
├── src/
│   ├── commands/        # Tauri command handlers
│   │   ├── mod.rs      # Module exports
│   │   ├── claude.rs   # Claude integration
│   │   ├── agents/     # Agent commands
│   │   └── ...         # Other commands
│   ├── db/             # Database layer
│   ├── sandbox/        # Sandboxing system
│   ├── checkpoint/     # Checkpoint system
│   ├── state.rs        # Application state
│   └── main.rs         # Entry point
├── tests/              # Integration tests
└── Cargo.toml          # Rust dependencies
```

## Code Style Guidelines

### TypeScript/React

1. **Component Structure**
   ```tsx
   import { useState } from 'react';
   import { ComponentProps } from './types';
   
   export function Component({ prop1, prop2 }: ComponentProps) {
     const [state, setState] = useState();
     
     // Event handlers
     const handleClick = () => {
       // Implementation
     };
     
     // Render
     return (
       <div>
         {/* Component content */}
       </div>
     );
   }
   ```

2. **File Naming**
   - Components: `PascalCase.tsx`
   - Utilities: `camelCase.ts`
   - Types: `types.ts` or `ComponentName.types.ts`
   - Tests: `ComponentName.test.tsx`

3. **Imports Order**
   ```typescript
   // 1. External dependencies
   import React from 'react';
   import { useQuery } from 'swr';
   
   // 2. Internal dependencies
   import { Button } from '@/components/ui';
   import { api } from '@/lib/api';
   
   // 3. Types
   import type { ComponentProps } from './types';
   
   // 4. Styles
   import './styles.css';
   ```

### Rust

1. **Module Organization**
   ```rust
   // Clear module structure
   mod error;
   mod types;
   mod handlers;
   
   pub use error::Error;
   pub use types::*;
   pub use handlers::*;
   ```

2. **Error Handling**
   ```rust
   use anyhow::Result;
   
   #[tauri::command]
   async fn command_name(arg: String) -> Result<String> {
       // Use ? for error propagation
       let result = some_operation()?;
       Ok(result)
   }
   ```

3. **Documentation**
   ```rust
   /// Brief description of the function.
   /// 
   /// # Arguments
   /// 
   /// * `arg` - Description of argument
   /// 
   /// # Returns
   /// 
   /// Description of return value
   /// 
   /// # Errors
   /// 
   /// This function will return an error if...
   pub fn function_name(arg: Type) -> Result<ReturnType> {
       // Implementation
   }
   ```

## Testing

### Frontend Testing

We use Vitest with React Testing Library:

```typescript
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Component } from './Component';

describe('Component', () => {
  it('renders correctly', () => {
    render(<Component />);
    expect(screen.getByText('Expected Text')).toBeInTheDocument();
  });
  
  it('handles user interaction', async () => {
    const { user } = render(<Component />);
    await user.click(screen.getByRole('button'));
    expect(screen.getByText('Updated Text')).toBeInTheDocument();
  });
});
```

### Backend Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function_name() {
        let result = function_name("input");
        assert_eq!(result.unwrap(), "expected output");
    }
    
    #[tokio::test]
    async fn test_async_function() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### Integration Testing

For testing Tauri commands:

```rust
#[cfg(test)]
mod integration_tests {
    use tauri::test::{mock_builder, MockRuntime};
    
    #[test]
    fn test_command() {
        let app = mock_builder().build(tauri::generate_context!()).unwrap();
        let window = app.get_window("main").unwrap();
        
        // Test command invocation
        // Assert results
    }
}
```

## Adding New Features

### 1. Plan the Feature
- Create an issue describing the feature
- Discuss implementation approach
- Consider UI/UX implications

### 2. Implement Backend
- Add new commands in `src-tauri/src/commands/`
- Update database schema if needed
- Write unit tests

### 3. Implement Frontend
- Create React components
- Add service layer functions
- Implement UI with shadcn/ui
- Write component tests

### 4. Integration
- Connect frontend to backend
- Test end-to-end flow
- Update documentation

### Example: Adding a New Command

**Backend** (`src-tauri/src/commands/feature.rs`):
```rust
use tauri::State;
use crate::AppState;

#[tauri::command]
pub async fn new_feature(
    state: State<'_, AppState>,
    param: String,
) -> Result<String, String> {
    // Implementation
    Ok("Result".to_string())
}
```

**Register Command** (`src-tauri/src/main.rs`):
```rust
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        // ... existing commands
        commands::feature::new_feature,
    ])
```

**Frontend** (`src/lib/api.ts`):
```typescript
export async function newFeature(param: string): Promise<string> {
  return invoke('new_feature', { param });
}
```

## Debugging

### Frontend Debugging

1. **Browser DevTools**
   - Use Chrome DevTools in development
   - React Developer Tools extension
   - Network tab for API calls

2. **Console Logging**
   ```typescript
   console.log('Debug info:', { data });
   console.table(arrayData);
   console.time('Operation');
   // ... code
   console.timeEnd('Operation');
   ```

3. **React DevTools Profiler**
   - Identify performance bottlenecks
   - Analyze component renders

### Backend Debugging

1. **Logging**
   ```rust
   use log::{debug, info, warn, error};
   
   debug!("Debug message: {:?}", data);
   info!("Operation completed");
   warn!("Warning: {}", message);
   error!("Error occurred: {}", err);
   ```

2. **VS Code Debugging**
   - Install CodeLLDB extension
   - Use provided launch configurations

3. **Print Debugging**
   ```rust
   dbg!(&variable);  // Prints file, line, and value
   println!("Value: {:?}", variable);
   ```

## Performance Optimization

### Frontend Performance

1. **Code Splitting**
   ```typescript
   const HeavyComponent = lazy(() => import('./HeavyComponent'));
   ```

2. **Memoization**
   ```typescript
   const MemoizedComponent = memo(Component);
   const memoizedValue = useMemo(() => computeExpensive(a, b), [a, b]);
   const memoizedCallback = useCallback(() => {}, [dependency]);
   ```

3. **Virtual Scrolling**
   - Use react-window for large lists
   - Implement pagination where appropriate

### Backend Performance

1. **Async Operations**
   ```rust
   use tokio::task;
   
   let handle = task::spawn(async {
       // Expensive operation
   });
   ```

2. **Connection Pooling**
   - Database connections are pooled
   - Reuse prepared statements

3. **Efficient Algorithms**
   - Use appropriate data structures
   - Avoid unnecessary allocations

## Contributing Workflow

1. **Create a Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make Changes**
   - Write code following style guidelines
   - Add tests for new functionality
   - Update documentation

3. **Commit Messages**
   ```bash
   # Format: type(scope): description
   git commit -m "feat(agents): add batch execution support"
   
   # Types: feat, fix, docs, style, refactor, perf, test, chore
   ```

4. **Push and Create PR**
   ```bash
   git push origin feature/your-feature-name
   ```
   - Create pull request on GitHub
   - Fill out PR template
   - Link related issues

## Release Process

1. **Version Bump**
   - Update version in `package.json`
   - Update version in `src-tauri/Cargo.toml`
   - Update version in `src-tauri/tauri.conf.json`

2. **Changelog**
   - Update CHANGELOG.md
   - Follow Keep a Changelog format

3. **Testing**
   - Run full test suite
   - Manual testing on all platforms
   - Performance benchmarks

4. **Build**
   ```bash
   # Build for all platforms
   bun run tauri build
   ```

5. **Release**
   - Create GitHub release
   - Attach built artifacts
   - Publish release notes

## Common Issues

### Hot Reload Not Working
- Ensure Vite server is running
- Check for syntax errors
- Clear Vite cache: `rm -rf node_modules/.vite`

### Rust Compilation Errors
- Update Rust: `rustup update`
- Clean build: `cargo clean`
- Check feature flags

### Database Migration Issues
- Check migration files in order
- Verify schema changes
- Test on fresh database

## Resources

- [Tauri Documentation](https://tauri.app/v2/guides/)
- [React Documentation](https://react.dev/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [shadcn/ui Components](https://ui.shadcn.com/)

## Getting Help

- GitHub Issues for bugs
- Discussions for questions
- Discord community (coming soon)

## Related Documentation

- [Architecture Overview](ARCHITECTURE.md)
- [Testing Guide](testing/README.md)
- [API Reference](api/README.md)