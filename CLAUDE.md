# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Claudia is a Tauri 2 desktop application providing a GUI for Claude Code CLI. It enables users to create custom agents, manage interactive Claude Code sessions, run secure background agents, and more.

## Quick Command Reference

```bash
# Development
bun run tauri dev    # Start full application in development mode
bun run dev          # Start frontend dev server only

# Testing
bun run test         # Run frontend tests
cd src-tauri && cargo test  # Run backend tests

# Build
bun run tauri build  # Build the complete application
```

## Key Technical Details

- **Frontend**: React 18 + TypeScript + Vite 6 + Tailwind CSS v4
- **Backend**: Rust + Tauri 2 + SQLite
- **Package Manager**: Bun
- **Testing**: Vitest (frontend) + Rust test framework (backend)

## Important Implementation Notes

- The app expects Claude Code CLI to be installed and available in PATH
- All data is stored locally in `~/.claude/` and the app's data directory
- Database migrations are handled automatically on startup
- Process cleanup includes graceful shutdown with timeout fallbacks
- Sandbox violations are logged but don't stop execution by default
- MCP servers can be imported from Claude Desktop configuration
- Frontend uses SWR for data fetching with automatic revalidation
- Session events are isolated to prevent cross-contamination

## Documentation Structure

- **[Architecture Overview](docs/ARCHITECTURE.md)**: Detailed system architecture and design patterns
- **[Development Guide](docs/DEVELOPMENT.md)**: Setup, workflow, and contribution guidelines
- **[Installation Guide](docs/INSTALLATION.md)**: Platform-specific installation instructions
- **[Testing Documentation](docs/testing/README.md)**: Comprehensive testing guide

## Working with the Codebase

When making changes:
1. Follow existing patterns in the codebase
2. Ensure tests pass before committing
3. Update relevant documentation
4. Use descriptive commit messages
5. Keep changes focused and atomic