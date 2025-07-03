# API Module Structure

This directory contains the refactored modular API structure for the Claudia application. The API has been split from a monolithic file into logical modules for better maintainability and organization.

## Directory Structure

```
api/
├── types/           # TypeScript type definitions
│   ├── index.ts     # Re-exports all types
│   ├── projects.ts  # Project and session types
│   ├── claude.ts    # Claude core types
│   ├── agents.ts    # Agent types
│   ├── sandbox.ts   # Sandbox types
│   ├── usage.ts     # Usage tracking types
│   ├── checkpoint.ts # Checkpoint/timeline types
│   ├── mcp.ts       # MCP (Model Context Protocol) types
│   └── file-system.ts # File system types
│
├── modules/         # API implementation modules
│   ├── index.ts     # Re-exports all modules
│   ├── projects.ts  # Project management API
│   ├── claude.ts    # Claude core functionality API
│   ├── agents.ts    # Agent management API
│   ├── sandbox.ts   # Sandbox security API
│   ├── usage.ts     # Usage tracking API
│   ├── checkpoint.ts # Checkpoint/timeline API
│   ├── mcp.ts       # MCP server management API
│   ├── file-system.ts # File system operations API
│   └── screenshot.ts # Screenshot functionality API
│
└── README.md        # This file
```

## Backward Compatibility

The main `api.ts` file in the parent directory (`src/lib/api.ts`) maintains full backward compatibility by re-exporting all types and spreading all module methods into a single `api` object. This ensures that existing code continues to work without any changes.

```typescript
// Old usage (still works)
import { api, Project, Session } from '@/lib/api';
const projects = await api.listProjects();
```

## Module Organization

Each module is responsible for a specific domain:

- **projects**: Project and session management
- **claude**: Claude Code CLI integration and settings
- **agents**: AI agent creation, execution, and management
- **sandbox**: Security sandboxing and violation tracking
- **usage**: Token usage statistics and analytics
- **checkpoint**: Session timeline and checkpoint management
- **mcp**: Model Context Protocol server management
- **file-system**: File and directory operations
- **screenshot**: Screenshot capture functionality

## Adding New Functionality

To add new API functionality:

1. **Add types**: Define interfaces in the appropriate file under `types/`
2. **Implement methods**: Add methods to the relevant module under `modules/`
3. **Export types**: Ensure new types are exported from `types/index.ts`
4. **Test**: The existing test structure in `api.test.ts` will automatically test the new methods

## Migration Notes

This refactoring introduced the following type renames to avoid conflicts:
- `ImportResult` → `SandboxImportResult` (sandbox module)
- `ImportResult` → `MCPImportResult` (MCP module)

All other types and method signatures remain unchanged.