# Development Documentation

This section contains guides and information for developers working on Claudia.

## Contents

### Getting Started
- [Quick Start Guide](./getting-started.md) - Set up and run Claudia
- [Development Environment](./environment-setup.md) - Tools and prerequisites
- [Project Structure](./project-structure.md) - Repository organization

### Development Guides
- [Frontend Development](./frontend-development.md) - React and TypeScript development
- [Backend Development](./backend-development.md) - Rust and Tauri development
- [Database Development](./database-development.md) - Working with SQLite
- [Testing Guide](./testing-guide.md) - Writing and running tests

### Build & Deployment
- [Build Process](./build-process.md) - Building for development and production
- [Release Process](./release-process.md) - Creating releases
- [Platform-Specific Notes](./platform-notes.md) - OS-specific considerations

### Contributing
- [Contributing Guidelines](./contributing.md) - How to contribute
- [Code Style Guide](./code-style.md) - Coding standards and conventions
- [Pull Request Process](./pr-process.md) - Submitting changes
- [Issue Guidelines](./issue-guidelines.md) - Reporting bugs and features

### Tooling & Configuration
- [Development Commands](./commands.md) - Complete command reference for development, testing, and building
- [IDE Setup](./ide-setup.md) - Recommended IDE configurations
- [Debugging](./debugging.md) - Debugging techniques

## Quick Commands

```bash
# Development
bun run dev          # Frontend dev server
bun run tauri dev    # Full Tauri app

# Testing
bun run test         # Frontend tests
cd src-tauri && cargo test  # Backend tests

# Building
bun run build        # Frontend build
bun run tauri build  # Full app build
```

## Prerequisites

- Node.js 18+ or Bun
- Rust 1.70+
- Platform-specific build tools

See [Development Environment](./environment-setup.md) for detailed setup instructions.