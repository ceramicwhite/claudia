# API Reference Documentation

This section provides comprehensive API documentation for Claudia's frontend and backend interfaces.

## Contents

### Frontend APIs
- [React Components](./components/) - Component API reference
- [Hooks](./hooks.md) - Custom React hooks
- [Services](./services.md) - Service layer APIs
- [Types & Interfaces](./types.md) - TypeScript type definitions
- [Utilities](./utilities.md) - Helper functions and utilities

### Backend APIs
- [Tauri Commands](./commands/) - Backend command reference
- [Database API](./database.md) - Database operations
- [Agent API](./agent-api.md) - Agent management functions
- [Process API](./process-api.md) - Process management
- [Sandbox API](./sandbox-api.md) - Sandboxing operations

### Integration APIs
- [Claude Integration](./claude-integration.md) - Claude Code CLI integration
- [MCP Integration](./mcp-api.md) - Model Context Protocol
- [Event System](./events.md) - Tauri event documentation

### Data Structures
- [Agent Schema](./schemas/agent.md) - Agent data structures
- [Session Schema](./schemas/session.md) - Session data structures
- [Project Schema](./schemas/project.md) - Project configuration
- [Database Schema](./schemas/database.md) - SQLite table definitions

## API Categories

### Core APIs
Essential APIs for basic functionality:
- Session management
- Agent execution
- File operations
- Process control

### Extension APIs
APIs for extended functionality:
- MCP server management
- Checkpoint system
- Usage analytics
- Sandbox profiles

### Internal APIs
Lower-level APIs (use with caution):
- Database connections
- Process registry
- Event emitters
- Cache management

## Quick Reference

### Common Frontend Patterns
```typescript
// Using Tauri commands
import { invoke } from '@tauri-apps/api/core';
const result = await invoke('command_name', { arg: value });

// Using services
import { AgentService } from '@/services';
const agents = await AgentService.list();
```

### Common Backend Patterns
```rust
// Defining commands
#[tauri::command]
async fn my_command(arg: String) -> Result<String, String> {
    // Implementation
}

// Database operations
let conn = pool.get()?;
let result = conn.execute(query, params)?;
```