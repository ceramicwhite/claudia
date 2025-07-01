# Claudia Architecture Overview

Claudia is a Tauri 2 desktop application providing a GUI for Claude Code CLI. This document consolidates the architecture information from across the codebase.

## System Overview

Claudia consists of three main layers:

1. **Frontend**: React + TypeScript web application
2. **Backend**: Rust + Tauri native layer
3. **Database**: SQLite for persistent storage

The application follows a command-based architecture where the frontend invokes Rust commands through Tauri's IPC bridge, and the backend manages processes, database operations, and system interactions.

## Frontend Architecture

### Technology Stack
- **Build Tool**: Vite 6 with Bun package manager
- **UI Framework**: Tailwind CSS v4 + shadcn/ui components  
- **Key Libraries**: React Hook Form, Framer Motion, Radix UI, Recharts, SWR
- **Language**: TypeScript with strict type checking

### Frontend Structure
```
src/
├── components/     # React components organized by feature
├── services/       # API service layer for data fetching
├── lib/           
│   └── api.ts     # Tauri command invocation wrapper
├── hooks/         # Custom React hooks
├── schemas/       # Zod schemas for type validation
└── App.tsx        # Main application routing and layout
```

### Key Frontend Patterns
- **Service Layer**: All Tauri commands go through service classes that handle validation and error transformation
- **SWR for Data Fetching**: Automatic revalidation and caching of backend data
- **Type Safety**: Zod schemas validate all data from the backend
- **Component Organization**: Features are grouped together with their related components

## Backend Architecture

### Technology Stack
- **Language**: Rust with async/await
- **Database**: SQLite via rusqlite with r2d2 connection pooling
- **Security**: Platform-specific sandboxing (gaol on Unix, custom on macOS)
- **Process Management**: Tokio for async runtime, custom process registry

### Backend Structure
```
src-tauri/src/
├── commands/       # Tauri command implementations
│   ├── agents/     # Agent management module
│   ├── claude.rs   # Claude Code integration
│   ├── mcp.rs      # Model Context Protocol servers
│   ├── sandbox.rs  # Security profiles
│   ├── usage.rs    # Token usage analytics
│   └── checkpoint.rs # Timeline versioning
├── db/            # Database utilities
├── state/         # Application state management
└── main.rs        # Application entry point
```

### Command Modules

#### agents/ - Custom AI Agents
The refactored agents module follows clean architecture principles:

```
agents/
├── mod.rs       # Main module with database init and core commands
├── error.rs     # Custom error types with thiserror
├── constants.rs # Pricing and configuration constants
├── types.rs     # Domain types and structs
├── helpers.rs   # Utility functions
├── execute.rs   # Agent execution logic
├── repository.rs # Database operations
├── service.rs   # Business logic layer
├── pool.rs      # Connection pool management
└── commands.rs  # Tauri command interface
```

Key improvements in the agents module:
- **Error Handling**: Custom `AgentError` enum with proper error variants
- **Type Safety**: Newtype wrappers for IDs, enums for status/models
- **Repository Pattern**: Clean separation of database operations
- **Service Layer**: Business logic separated from commands
- **Constants**: Centralized pricing and configuration

#### claude.rs - Claude Code Integration
- Manages Claude Code CLI process lifecycle
- Handles session management and streaming output
- Provides commands for execute, continue, resume, cancel

#### mcp.rs - Model Context Protocol
- Manages MCP server lifecycles
- Imports configurations from Claude Desktop
- Provides server start/stop/restart capabilities

#### sandbox.rs - Security Profiles
- Defines security rules for agent execution
- Platform-specific sandbox implementations
- Tracks and reports sandbox violations

## Key Architectural Patterns

### 1. Process Management
- **Global State**: `ClaudeProcessState` for active Claude sessions
- **Process Registry**: Manages multiple concurrent agent executions
- **Streaming Output**: Real-time output via Tauri events with session isolation
- **Graceful Shutdown**: Timeout-based cleanup with fallback termination

### 2. Checkpoint System
- **Timeline-based Versioning**: Each session maintains a timeline of checkpoints
- **Content-addressable Storage**: File snapshots stored by content hash
- **Fork/Branch Support**: Exploration paths for different approaches
- **Efficient Storage**: Deduplication of identical file content

### 3. Frontend-Backend Communication
- **Command Pattern**: `await invoke<T>("command_name", args)`
- **Event System**: Real-time updates via `listen("event-name", callback)`
- **Session Isolation**: Events are scoped to prevent cross-session contamination
- **Type Safety**: Commands and events are strongly typed on both sides

### 4. Database Design
- **Connection Pooling**: r2d2 pool with appropriate sizing
- **Migration System**: Automatic schema updates on startup
- **Foreign Keys**: Enforced referential integrity
- **Indexes**: Optimized queries for common access patterns

## Security Model

### Sandboxing Approach
1. **Agent-specific Profiles**: Each agent has configurable permissions
2. **Platform Detection**: Uses best available sandboxing per platform
3. **Graceful Degradation**: Falls back safely when sandboxing unavailable
4. **Violation Logging**: Records but doesn't block on violations

### Permission Types
- **File Read**: Access to project files
- **File Write**: Modify project files
- **Network**: Internet access for API calls
- **System**: Access to system binaries and tools

### Platform-specific Implementation
- **macOS**: Custom sandbox using sandbox-exec
- **Linux**: Namespace isolation with gaol
- **Windows**: Limited sandboxing with process restrictions

## Data Flow

### Agent Execution Flow
1. Frontend requests agent execution with task
2. Backend creates database run record
3. Process spawned with sandbox profile
4. Output streamed via JSONL parsing
5. Metrics calculated and stored
6. Frontend updated via events

### Session Management Flow
1. User opens project session
2. Backend creates session record
3. Claude process started with session ID
4. Messages tracked in database
5. Checkpoint created on significant changes
6. Session can be resumed from any checkpoint

## Implementation Notes

### Critical Considerations
- Claude Code CLI must be installed and available in PATH
- All data stored locally in `~/.claude/` and app data directory
- Database migrations handled automatically on startup
- Process cleanup includes graceful shutdown with timeout fallbacks
- Sandbox violations logged but don't stop execution by default
- MCP servers can be imported from Claude Desktop configuration
- Frontend uses SWR for data fetching with automatic revalidation
- Testing includes both Vitest for frontend and cargo test for backend

### Performance Optimizations
- Connection pooling for database access
- Lazy loading of session history
- Event debouncing for real-time updates
- Efficient JSONL parsing with streaming
- Content-addressable storage reduces duplication

### Error Handling Strategy
- Backend errors transformed to user-friendly messages
- Retry logic for transient failures
- Graceful degradation for missing features
- Comprehensive error logging for debugging

## Future Architecture Considerations

### Planned Enhancements
1. **Plugin System**: Allow third-party agent extensions
2. **Remote Execution**: Optional cloud-based agent runs
3. **Collaborative Features**: Share agents and sessions
4. **Advanced Scheduling**: Cron-based agent execution
5. **Enhanced Security**: More granular permission controls

### Scalability Considerations
- Database sharding for large deployments
- Distributed process execution
- Caching layer for frequently accessed data
- Optimization of checkpoint storage