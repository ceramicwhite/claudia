# Claudia Architecture

This document provides a comprehensive overview of Claudia's architecture, design patterns, and key implementation details.

## Overview

Claudia is a Tauri 2 desktop application providing a GUI for Claude Code CLI. It follows a clean separation between the frontend (React + TypeScript) and backend (Rust + Tauri), with secure process management and sandboxing capabilities.

## Tech Stack

- **Frontend**: React 18 + TypeScript + Vite 6
- **Backend**: Rust with Tauri 2
- **UI Framework**: Tailwind CSS v4 + shadcn/ui
- **Database**: SQLite (via rusqlite with r2d2 connection pooling)
- **Package Manager**: Bun

## Project Structure

```
claudia/
├── src/                   # React frontend
│   ├── components/        # UI components organized by feature
│   ├── services/          # API service layer for data fetching
│   ├── lib/               # Utilities and API client
│   │   └── api.ts         # Tauri command invocation wrapper
│   ├── assets/            # Static assets
│   └── App.tsx            # Main application routing and layout
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── commands/      # Tauri command handlers
│   │   │   ├── claude.rs  # Claude Code integration
│   │   │   ├── agents/    # Custom AI agents
│   │   │   ├── mcp.rs     # MCP server management
│   │   │   ├── sandbox.rs # Security profiles
│   │   │   ├── usage.rs   # Token usage analytics
│   │   │   └── checkpoint.rs # Timeline versioning
│   │   ├── sandbox/       # Security sandboxing implementation
│   │   └── checkpoint/    # Timeline management system
│   └── tests/             # Rust test suite
└── public/                # Public assets
```

## Frontend Architecture

### Build Tool Configuration
- **Vite 6**: Fast HMR and optimized builds
- **TypeScript**: Strict type checking enabled
- **Bun**: Fast package management and script execution

### Component Organization
- Feature-based component structure
- Shared components in `components/ui/`
- Context providers for global state
- Custom hooks for business logic

### Key Libraries
- **React Hook Form**: Form state management
- **Framer Motion**: Animations and transitions
- **Radix UI**: Accessible component primitives
- **Recharts**: Data visualization
- **SWR**: Data fetching with caching

### State Management
- React Context for global state
- SWR for server state caching
- Local state with useState/useReducer
- Session isolation through event namespacing

## Backend Architecture

### Core Modules

#### Process Management
- **ClaudeProcessState**: Global state for active Claude sessions
- **ProcessRegistry**: Manages multiple agent executions
- Session-isolated event streaming
- Graceful shutdown with timeout fallbacks

#### Security & Sandboxing
- Platform-specific implementations:
  - Linux: seccomp-based sandboxing
  - macOS: Seatbelt (sandbox-exec) profiles
  - Windows: Graceful degradation
- Rule-based permission system
- Violation tracking and audit logging
- Reusable security profiles

#### Database Layer
- SQLite with connection pooling (r2d2)
- Automatic schema migrations on startup
- Transaction support for data integrity
- Prepared statements for performance

#### Command Modules
1. **claude.rs**: Claude Code CLI integration
   - Session management
   - Process lifecycle control
   - Output streaming

2. **agents/**: Custom AI agent system
   - Agent creation and configuration
   - Execution tracking
   - Sandboxed execution environment

3. **mcp.rs**: Model Context Protocol
   - Server registration and management
   - Configuration import/export
   - Connection testing

4. **checkpoint.rs**: Timeline system
   - Content-addressable storage
   - Fork/branch support
   - Diff generation

5. **usage.rs**: Analytics and tracking
   - Token usage monitoring
   - Cost calculation
   - Usage history

### Checkpoint System

The checkpoint system provides Git-like versioning for coding sessions:

1. **Timeline Management**
   - Linear history with branching support
   - Parent-child relationships
   - Metadata preservation

2. **Content Storage**
   - SHA-256 based content addressing
   - Efficient deduplication
   - Binary and text file support

3. **Operations**
   - Create checkpoints at any point
   - Restore to previous states
   - Fork from existing checkpoints
   - Diff between checkpoints

## Frontend-Backend Communication

### Command Invocation
```typescript
// Frontend
await invoke<T>("command_name", { arg1: value1, arg2: value2 })

// Backend
#[tauri::command]
async fn command_name(arg1: Type1, arg2: Type2) -> Result<T, Error>
```

### Event System
```typescript
// Frontend - Listen for events
const unlisten = await listen("event-name", (event) => {
  console.log(event.payload);
});

// Backend - Emit events
app_handle.emit("event-name", payload)?;
```

### Session Isolation
- Events are namespaced by session ID
- Prevents cross-contamination between sessions
- Automatic cleanup on session end

## Security Architecture

### Multi-Layer Security Model

1. **Process Isolation**
   - Each agent runs in a separate process
   - Limited inter-process communication
   - Resource limits enforced

2. **Filesystem Access Control**
   - Whitelist-based permissions
   - Path normalization and validation
   - Symlink resolution

3. **Network Restrictions**
   - Configurable network access
   - DNS filtering support
   - Connection logging

4. **System Call Filtering**
   - Platform-specific implementations
   - Minimal required syscall set
   - Violation logging

### Sandbox Profiles

Profiles define security boundaries:
```rust
pub struct SandboxProfile {
    pub name: String,
    pub description: String,
    pub rules: Vec<SandboxRule>,
    pub platform_specific: PlatformConfig,
}
```

### Audit Trail
- All violations logged to database
- Real-time monitoring UI
- Export capabilities for compliance

## Data Storage

### Local Storage Locations
- **User Data**: `~/.claude/` directory
- **App Data**: Platform-specific app directories
- **Database**: SQLite file in app data
- **Checkpoints**: Content-addressed storage

### Database Schema
- Normalized design with foreign keys
- Indexes on frequently queried columns
- Views for complex queries
- Triggers for audit trails

## Performance Considerations

### Frontend Optimization
- Code splitting with dynamic imports
- Lazy loading of heavy components
- Memoization of expensive computations
- Virtual scrolling for large lists

### Backend Optimization
- Connection pooling for database
- Prepared statement caching
- Async command handlers
- Efficient file streaming

### Process Management
- Process pooling for agents
- Output buffering and batching
- Graceful degradation under load
- Resource cleanup on exit

## Error Handling

### Frontend Error Boundaries
- Component-level error boundaries
- Fallback UI for errors
- Error reporting to backend
- User-friendly error messages

### Backend Error Strategy
- Result<T, E> for all operations
- Custom error types with context
- Error propagation with `?` operator
- Structured error responses

## Testing Architecture

### Frontend Testing
- Vitest for unit and integration tests
- React Testing Library for components
- MSW for API mocking
- Coverage reporting

### Backend Testing
- Cargo test framework
- Unit tests for commands
- Integration tests for system
- Property-based testing for algorithms

## Future Architectural Considerations

### Planned Improvements
1. Plugin system for extensibility
2. Remote agent execution
3. Collaborative features
4. Cloud sync capabilities

### Scalability Paths
- Microservice extraction for heavy operations
- Background job queue system
- Caching layer for expensive operations
- Horizontal scaling for agent execution

## Related Documentation

- [DEVELOPMENT.md](DEVELOPMENT.md) - Development setup and workflow
- [INSTALLATION.md](INSTALLATION.md) - Installation instructions
- [Testing Documentation](testing/README.md) - Comprehensive testing guide
- [Agent Module Architecture](architecture/agent-module-readme.md) - Deep dive into agents