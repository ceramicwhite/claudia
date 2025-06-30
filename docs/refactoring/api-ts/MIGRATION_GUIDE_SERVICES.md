# Service Architecture Migration Guide

## Overview

The API module has been refactored from a single monolithic `api.ts` file (2030 lines) into domain-specific service modules. This improves code organization, maintainability, and type safety.

## Service Structure

### Base Service
All services extend from `BaseService` which provides:
- Type-safe `invoke` wrapper with centralized error handling
- `invokeWithAutoError` for automatic error message generation
- `safeInvoke` for operations that should return defaults instead of throwing

### Domain Services

1. **ProjectService** (`project.service.ts`)
   - Project listing and management
   - Claude settings operations
   - File and directory operations
   - Claude installation management

2. **SessionService** (`session.service.ts`)
   - Session retrieval and creation
   - Session history management
   - Message tracking for checkpoints

3. **ClaudeService** (`claude.service.ts`)
   - Claude Code execution
   - Session continuation and resumption
   - Execution cancellation

4. **AgentService** (`agent.service.ts`)
   - Agent CRUD operations
   - Agent execution and scheduling
   - GitHub agent integration
   - Running session management
   - Output streaming

5. **SandboxService** (`sandbox.service.ts`)
   - Sandbox profile management
   - Rule configuration
   - Violation tracking
   - Import/export functionality

6. **UsageService** (`usage.service.ts`)
   - Usage statistics and analytics
   - Date range filtering
   - Session-based metrics

7. **CheckpointService** (`checkpoint.service.ts`)
   - Checkpoint creation and restoration
   - Timeline management
   - Auto-checkpoint configuration
   - Diff generation

8. **MCPService** (`mcp.service.ts`)
   - MCP server management
   - Configuration import/export
   - Connection testing
   - Project configuration

## Migration Steps

### 1. Update Imports

**Before:**
```typescript
import { api } from '@/lib/api';

// Usage
const projects = await api.listProjects();
```

**After (Recommended):**
```typescript
import { projectService } from '@/services';

// Usage
const projects = await projectService.listProjects();
```

**Alternative (using specific service):**
```typescript
import { projectService } from '@/services/project.service';

// Usage
const projects = await projectService.listProjects();
```

### 2. Type Imports

Types can be imported from either location:

```typescript
// From services index (recommended)
import type { Project, Agent, UsageStats } from '@/services';

// Or from api.ts (for compatibility)
import type { Project, Agent, UsageStats } from '@/lib/api';
```

### 3. Component Migration Example

**Before:**
```typescript
import { api } from '@/lib/api';
import type { Agent } from '@/lib/api';

const AgentList = () => {
  const [agents, setAgents] = useState<Agent[]>([]);
  
  useEffect(() => {
    api.listAgents().then(setAgents);
  }, []);
  
  const handleCreate = async (data: AgentData) => {
    const agent = await api.createAgent(
      data.name,
      data.icon,
      data.systemPrompt
    );
    // ...
  };
};
```

**After:**
```typescript
import { agentService } from '@/services';
import type { Agent } from '@/services';

const AgentList = () => {
  const [agents, setAgents] = useState<Agent[]>([]);
  
  useEffect(() => {
    agentService.listAgents().then(setAgents);
  }, []);
  
  const handleCreate = async (data: AgentData) => {
    const agent = await agentService.createAgent(
      data.name,
      data.icon,
      data.systemPrompt
    );
    // ...
  };
};
```

## Benefits

1. **Better Organization**: Related methods are grouped together
2. **Improved Discoverability**: Service names indicate their purpose
3. **Type Safety**: Each service has focused type imports
4. **Maintainability**: Smaller, focused files are easier to maintain
5. **Testing**: Services can be mocked individually
6. **Performance**: Potential for better tree-shaking

## Backward Compatibility

The original `api` object is maintained for backward compatibility but is marked as deprecated. It proxies all calls to the appropriate services.

```typescript
// This still works but is deprecated
import { api } from '@/lib/api';
await api.listProjects();

// Preferred approach
import { projectService } from '@/services';
await projectService.listProjects();
```

## Future Considerations

1. **Gradual Migration**: Components can be migrated incrementally
2. **Testing**: Update tests to use service imports
3. **Error Handling**: Services provide consistent error handling
4. **Extension**: New methods should be added to appropriate services

## Service Method Mapping

| Original api.method | New Service |
|-------------------|-------------|
| listProjects | projectService |
| getProjectSessions | sessionService |
| executeClaudeCode | claudeService |
| listAgents | agentService |
| listSandboxProfiles | sandboxService |
| getUsageStats | usageService |
| createCheckpoint | checkpointService |
| mcpList | mcpService |

See individual service files for complete method listings.