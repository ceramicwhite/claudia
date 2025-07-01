# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Development
bun run dev          # Start Vite frontend dev server only
bun run tauri dev    # Start full Tauri application in development mode

# Build
bun run build        # TypeScript check + Vite build (frontend only)
bun run tauri build  # Build the complete Tauri application

# Testing
bun run test         # Run frontend tests in watch mode
bun run test:ui      # Run tests with UI
bun run test:run     # Run tests once
bun run test:coverage # Generate coverage report
cd src-tauri && cargo test  # Run Rust backend tests

# Code Formatting
cd src-tauri && cargo fmt   # Format Rust code

# Preview
bun run preview      # Preview production build
```

## Architecture Overview

Claudia is a Tauri 2 desktop application providing a GUI for Claude Code CLI. It consists of:

### Frontend (React + TypeScript)
- **Build Tool**: Vite 6 with Bun package manager
- **UI Framework**: Tailwind CSS v4 + shadcn/ui components
- **Key Libraries**: React Hook Form, Framer Motion, Radix UI, Recharts, SWR
- **Structure**:
  - `src/components/` - React components organized by feature
  - `src/services/` - API service layer for data fetching
  - `src/lib/api.ts` - Tauri command invocation wrapper
  - `src/App.tsx` - Main application routing and layout

### Backend (Rust + Tauri)
- **Database**: SQLite via rusqlite with r2d2 connection pooling
- **Security**: Platform-specific sandboxing (gaol on Unix, custom on macOS)
- **Command Modules** in `src-tauri/src/commands/`:
  - `claude.rs` - Claude Code integration and session management
  - `agents/` - Custom AI agents with execution tracking and sandboxing
  - `mcp.rs` - Model Context Protocol server management
  - `sandbox.rs` - Security profiles and violation tracking
  - `usage.rs` - Token usage analytics
  - `checkpoint.rs` - Timeline and session versioning

### Key Architectural Patterns

1. **Process Management**: 
   - Global `ClaudeProcessState` for active Claude sessions
   - `ProcessRegistry` for multiple agent executions
   - Streaming output via Tauri events with session isolation

2. **Checkpoint System**:
   - Timeline-based session versioning in `checkpoint/` module
   - Content-addressable storage for file snapshots
   - Fork/branch support for exploration paths

3. **Frontend-Backend Communication**:
   - Tauri `invoke` for commands: `await invoke<T>("command_name", args)`
   - Event system for real-time updates: `listen("event-name", callback)`
   - Session-isolated events prevent cross-contamination

4. **Security Model**:
   - Agent-specific sandbox profiles with rule-based permissions
   - Platform detection for available security features
   - Graceful degradation when sandboxing unavailable

## Important Implementation Notes

- The app expects Claude Code CLI to be installed and available in PATH
- All data is stored locally in `~/.claude/` and the app's data directory
- Database migrations are handled automatically on startup
- Process cleanup includes graceful shutdown with timeout fallbacks
- Sandbox violations are logged but don't stop execution by default
- MCP servers can be imported from Claude Desktop configuration
- Frontend uses SWR for data fetching with automatic revalidation
- Testing setup includes both Vitest for frontend and cargo test for backend