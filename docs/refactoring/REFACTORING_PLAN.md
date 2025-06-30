# Claudia Refactoring Plan

## Executive Summary

This comprehensive refactoring plan addresses the identified issues in the Claudia codebase:
- **api.ts** (2030 lines) → split into 8 service modules
- **ToolWidgets.tsx** (2290 lines) → split into 20+ widget components
- **Backend command modules** (7590 lines total) → modularize large files
- **Type safety** improvements across the entire codebase
- **Performance** optimizations with React memoization

## Dependency Analysis

### Critical Dependencies Map

```
Frontend Dependencies:
├── api.ts (core)
│   ├── All components depend on this
│   ├── No internal dependencies (only @tauri-apps/api)
│   └── 40+ exported interfaces/types
│
├── ToolWidgets.tsx
│   ├── Dependencies: lucide-react, react-markdown, react-syntax-highlighter
│   ├── Imported by: SessionOutputViewer, StreamMessage
│   └── 20+ widget components
│
└── Component Tree
    ├── App.tsx → Routes to all major views
    ├── ClaudeCodeSession.tsx → Central session management
    ├── AgentExecution.tsx → Agent runner
    └── Settings.tsx → Configuration hub

Backend Dependencies:
├── commands/claude.rs (1977 lines) → Core Claude integration
├── commands/agents.rs (2951 lines) → Agent management
├── commands/sandbox.rs (947 lines) → Security layer
└── commands/usage.rs (714 lines) → Analytics
```

### Circular Dependencies
- None detected in current codebase ✓
- Risk areas: If api.ts services reference each other

### Refactoring Order
1. Phase 1: Extract types/interfaces from api.ts
2. Phase 2: Split api.ts into services
3. Phase 3: Extract ToolWidgets components
4. Phase 4: Type safety improvements
5. Phase 5: Performance optimizations

## Risk Assessment

### High-Risk Areas
1. **Event System** (claude.rs → frontend)
   - Real-time streaming critical for UX
   - Session isolation must be maintained
   - Rollback: Keep original event handlers

2. **Process Management** (ClaudeProcessState)
   - Active sessions must not be interrupted
   - Rollback: Feature flag for new implementation

3. **Database Migrations**
   - Schema changes affect persistent data
   - Rollback: Backup before migration

### Medium-Risk Areas
1. **Type Changes** (any → unknown)
   - May break runtime behavior
   - Rollback: Progressive typing with tests

2. **Component Splitting**
   - Props drilling complexity
   - Rollback: Keep original as fallback

### Low-Risk Areas
1. **Dead code removal**
2. **Variable renaming**
3. **Extract constants**
4. **Code formatting**

## Phase 1: Safe Refactorings (Week 1)

### 1.1 Code Cleanup
```bash
# Remove console.logs
grep -r "console.log" src/ --include="*.ts" --include="*.tsx" | wc -l
# Current: ~50 occurrences

# Dead code analysis
# Remove: unused exports, commented code
```

### 1.2 Extract Constants
```typescript
// Before (scattered magic strings)
invoke("list_projects")
invoke("get_project_sessions")

// After (src/lib/constants.ts)
export const COMMANDS = {
  LIST_PROJECTS: 'list_projects',
  GET_PROJECT_SESSIONS: 'get_project_sessions',
  // ... all 100+ commands
} as const;

export const EVENTS = {
  CLAUDE_OUTPUT: 'claude-output',
  AGENT_OUTPUT: 'agent-output',
  // ... all events
} as const;
```

### 1.3 Naming Conventions
```typescript
// Standardize parameter names
projectId → projectId (not project_id in TS)
session_id → sessionId
download_url → downloadUrl
```

## Phase 2: Structural Improvements (Week 2-3)

### 2.1 Split api.ts into Services

```
src/lib/api/
├── index.ts           // Re-export for compatibility
├── types/             // All interfaces/types
│   ├── project.types.ts
│   ├── session.types.ts
│   ├── agent.types.ts
│   ├── sandbox.types.ts
│   ├── usage.types.ts
│   ├── checkpoint.types.ts
│   └── mcp.types.ts
│
├── services/
│   ├── base.service.ts      // Shared invoke wrapper
│   ├── project.service.ts   // Project operations
│   ├── session.service.ts   // Session management
│   ├── claude.service.ts    // Claude operations
│   ├── agent.service.ts     // Agent operations
│   ├── sandbox.service.ts   // Sandbox/security
│   ├── usage.service.ts     // Analytics/usage
│   ├── checkpoint.service.ts // Timeline/checkpoints
│   └── mcp.service.ts       // MCP server management
│
└── utils/
    ├── error-handler.ts     // Centralized error handling
    └── type-guards.ts       // Runtime type validation
```

#### Example Service Structure:
```typescript
// src/lib/api/services/base.service.ts
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { z } from "zod";

export class BaseService {
  protected async invoke<T>(
    command: string,
    args?: Record<string, unknown>,
    schema?: z.ZodSchema<T>
  ): Promise<T> {
    try {
      const result = await tauriInvoke<T>(command, args);
      return schema ? schema.parse(result) : result;
    } catch (error) {
      console.error(`Command ${command} failed:`, error);
      throw error;
    }
  }
}

// src/lib/api/services/project.service.ts
import { BaseService } from './base.service';
import { Project, Session } from '../types';
import { COMMANDS } from '@/lib/constants';

export class ProjectService extends BaseService {
  async listProjects(): Promise<Project[]> {
    return this.invoke<Project[]>(COMMANDS.LIST_PROJECTS);
  }

  async getProjectSessions(projectId: string): Promise<Session[]> {
    return this.invoke<Session[]>(
      COMMANDS.GET_PROJECT_SESSIONS,
      { projectId }
    );
  }
  
  // ... other project methods
}
```

### 2.2 Split ToolWidgets.tsx

```
src/components/widgets/
├── index.ts              // Re-export all widgets
├── base/
│   ├── WidgetContainer.tsx
│   ├── WidgetHeader.tsx
│   └── CodeBlock.tsx
│
├── todo/
│   ├── TodoWidget.tsx
│   └── TodoItem.tsx
│
├── file/
│   ├── LSWidget.tsx
│   ├── LSResultWidget.tsx
│   ├── ReadWidget.tsx
│   ├── ReadResultWidget.tsx
│   ├── WriteWidget.tsx
│   └── FileIcon.tsx
│
├── search/
│   ├── GrepWidget.tsx
│   ├── GlobWidget.tsx
│   └── SearchResult.tsx
│
├── edit/
│   ├── EditWidget.tsx
│   ├── EditResultWidget.tsx
│   ├── MultiEditWidget.tsx
│   ├── MultiEditResultWidget.tsx
│   └── DiffViewer.tsx
│
├── command/
│   ├── BashWidget.tsx
│   ├── CommandWidget.tsx
│   └── CommandOutputWidget.tsx
│
├── mcp/
│   └── MCPWidget.tsx
│
└── system/
    ├── SystemReminderWidget.tsx
    ├── SystemInitializedWidget.tsx
    ├── SummaryWidget.tsx
    └── TaskWidget.tsx
```

#### Example Refactored Widget:
```typescript
// src/components/widgets/todo/TodoWidget.tsx
import React from 'react';
import { CheckCircle2, Circle, Clock } from 'lucide-react';
import { cn } from '@/lib/utils';
import { TodoItem } from './TodoItem';
import { WidgetContainer, WidgetHeader } from '../base';

interface TodoWidgetProps {
  todos: Todo[];
  result?: unknown;
}

export const TodoWidget: React.FC<TodoWidgetProps> = ({ todos }) => {
  return (
    <WidgetContainer>
      <WidgetHeader icon="list-checks" title="Todo List" />
      <div className="space-y-2">
        {todos.map((todo, idx) => (
          <TodoItem key={todo.id || idx} todo={todo} />
        ))}
      </div>
    </WidgetContainer>
  );
};
```

### 2.3 Backend Modularization (Rust)

Split large command modules:

```
src-tauri/src/commands/
├── agents/
│   ├── mod.rs
│   ├── crud.rs         // Create, read, update, delete
│   ├── execution.rs    // Run management
│   ├── github.rs       // GitHub integration
│   └── scheduled.rs    // Scheduling features
│
├── claude/
│   ├── mod.rs
│   ├── process.rs      // Process management
│   ├── session.rs      // Session handling
│   ├── streaming.rs    // Output streaming
│   └── settings.rs     // Settings management
│
└── sandbox/
    ├── mod.rs
    ├── profiles.rs     // Profile CRUD
    ├── rules.rs        // Rule management
    └── violations.rs   // Violation tracking
```

## Phase 3: Pattern Migrations (Week 4-5)

### 3.1 Implement Zod Validation

```typescript
// src/lib/api/schemas/project.schema.ts
import { z } from 'zod';

export const ProjectSchema = z.object({
  id: z.string(),
  path: z.string(),
  sessions: z.array(z.string()),
  created_at: z.number(),
});

export const SessionSchema = z.object({
  id: z.string().uuid(),
  project_id: z.string(),
  project_path: z.string(),
  todo_data: z.any().optional(),
  created_at: z.number(),
  first_message: z.string().optional(),
  message_timestamp: z.string().optional(),
});

// Usage in service
async getProjectSessions(projectId: string): Promise<Session[]> {
  const result = await this.invoke(
    COMMANDS.GET_PROJECT_SESSIONS,
    { projectId }
  );
  return z.array(SessionSchema).parse(result);
}
```

### 3.2 Error Handling Pattern

```typescript
// src/lib/api/utils/error-handler.ts
export class ApiError extends Error {
  constructor(
    message: string,
    public code: string,
    public details?: unknown
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

export function handleApiError(error: unknown): never {
  if (error instanceof z.ZodError) {
    throw new ApiError(
      'Validation failed',
      'VALIDATION_ERROR',
      error.errors
    );
  }
  
  if (error instanceof Error) {
    throw new ApiError(
      error.message,
      'UNKNOWN_ERROR',
      error
    );
  }
  
  throw new ApiError(
    'An unexpected error occurred',
    'UNKNOWN_ERROR',
    error
  );
}
```

### 3.3 Context Pattern for State

```typescript
// src/contexts/SessionContext.tsx
interface SessionContextValue {
  activeSession: Session | null;
  isLoading: boolean;
  error: Error | null;
  startSession: (projectPath: string) => Promise<void>;
  stopSession: () => Promise<void>;
}

const SessionContext = createContext<SessionContextValue | null>(null);

export function SessionProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(sessionReducer, initialState);
  
  const value = useMemo(() => ({
    ...state,
    startSession: async (projectPath) => {
      // Implementation
    },
    stopSession: async () => {
      // Implementation
    }
  }), [state]);
  
  return (
    <SessionContext.Provider value={value}>
      {children}
    </SessionContext.Provider>
  );
}
```

## Phase 4: Type Safety (Week 6)

### 4.1 Replace `any` Types

```typescript
// Before
todo_data?: any;
result?: any;
files: any[];

// After
todo_data?: TodoData;
result?: OperationResult;
files: FileEntry[];

// Type guards
function isTodoData(value: unknown): value is TodoData {
  return (
    typeof value === 'object' &&
    value !== null &&
    'items' in value &&
    Array.isArray((value as any).items)
  );
}
```

### 4.2 Strict TypeScript Configuration

```json
{
  "compilerOptions": {
    "strict": true,
    "strictNullChecks": true,
    "strictFunctionTypes": true,
    "strictBindCallApply": true,
    "strictPropertyInitialization": true,
    "noImplicitAny": true,
    "noImplicitThis": true,
    "useUnknownInCatchVariables": true,
    "noUncheckedIndexedAccess": true
  }
}
```

### 4.3 Generic Constraints

```typescript
// Constrained generics for better inference
export function createService<T extends BaseService>(
  ServiceClass: new () => T
): T {
  return new ServiceClass();
}

// Type-safe event emitter
export class TypedEventEmitter<TEvents extends Record<string, any>> {
  on<K extends keyof TEvents>(
    event: K,
    handler: (data: TEvents[K]) => void
  ): void {
    // Implementation
  }
}
```

## Phase 5: Performance Optimizations (Week 7)

### 5.1 React Memoization

```typescript
// Memoize expensive computations
export const SessionList = memo(({ sessions }: Props) => {
  const sortedSessions = useMemo(
    () => sessions.sort((a, b) => b.created_at - a.created_at),
    [sessions]
  );
  
  const handleSessionClick = useCallback((id: string) => {
    // Handle click
  }, []);
  
  return (
    <div>
      {sortedSessions.map(session => (
        <SessionCard
          key={session.id}
          session={session}
          onClick={handleSessionClick}
        />
      ))}
    </div>
  );
});

// Memoize context values
const contextValue = useMemo(() => ({
  state,
  actions: {
    startSession,
    stopSession,
  }
}), [state, startSession, stopSession]);
```

### 5.2 Lazy Loading

```typescript
// Lazy load heavy components
const UsageDashboard = lazy(() => import('./components/UsageDashboard'));
const CheckpointSettings = lazy(() => import('./components/CheckpointSettings'));

// Split widget imports
const widgetComponents = {
  todo: lazy(() => import('./widgets/todo/TodoWidget')),
  ls: lazy(() => import('./widgets/file/LSWidget')),
  // ... other widgets
};
```

### 5.3 Virtual Scrolling

```typescript
// Use @tanstack/react-virtual for long lists
import { useVirtualizer } from '@tanstack/react-virtual';

export function VirtualSessionList({ sessions }: Props) {
  const parentRef = useRef<HTMLDivElement>(null);
  
  const virtualizer = useVirtualizer({
    count: sessions.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 80,
  });
  
  return (
    <div ref={parentRef} className="h-full overflow-auto">
      <div style={{ height: virtualizer.getTotalSize() }}>
        {virtualizer.getVirtualItems().map(virtualItem => (
          <div
            key={virtualItem.key}
            style={{
              position: 'absolute',
              top: 0,
              left: 0,
              width: '100%',
              transform: `translateY(${virtualItem.start}px)`,
            }}
          >
            <SessionCard session={sessions[virtualItem.index]} />
          </div>
        ))}
      </div>
    </div>
  );
}
```

## Testing Strategy

### 1. Characterization Tests
```typescript
// Capture current behavior before refactoring
describe('api.listProjects', () => {
  it('should return projects in expected format', async () => {
    const projects = await api.listProjects();
    expect(projects).toMatchSnapshot();
  });
});
```

### 2. Integration Tests
```typescript
// Test service integration
describe('ProjectService', () => {
  let service: ProjectService;
  
  beforeEach(() => {
    service = new ProjectService();
  });
  
  it('should handle errors gracefully', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('Network error'));
    
    await expect(service.listProjects()).rejects.toThrow('Network error');
  });
});
```

### 3. Component Tests
```typescript
// Test refactored widgets
describe('TodoWidget', () => {
  it('should render todos correctly', () => {
    const todos = [
      { id: '1', content: 'Test', status: 'pending', priority: 'high' }
    ];
    
    render(<TodoWidget todos={todos} />);
    
    expect(screen.getByText('Test')).toBeInTheDocument();
    expect(screen.getByText('high')).toBeInTheDocument();
  });
});
```

## Rollback Strategies

### 1. Feature Flags
```typescript
// Enable progressive rollout
export const FEATURES = {
  USE_NEW_API: process.env.USE_NEW_API === 'true',
  USE_SPLIT_WIDGETS: process.env.USE_SPLIT_WIDGETS === 'true',
};

// Usage
const api = FEATURES.USE_NEW_API ? newApi : oldApi;
```

### 2. Parallel Implementation
```typescript
// Keep old implementation alongside new
export { api as legacyApi } from './api.legacy';
export { api } from './api';
```

### 3. Database Backups
```bash
# Before migrations
cp ~/.claude/claudia.db ~/.claude/claudia.db.backup

# Rollback script
mv ~/.claude/claudia.db.backup ~/.claude/claudia.db
```

## Success Metrics

1. **Code Quality**
   - Reduce file sizes: api.ts < 250 lines per service
   - Type coverage: > 95%
   - Test coverage: > 80%

2. **Performance**
   - Initial load time: < 2s
   - Session switching: < 100ms
   - Memory usage: < 20% reduction

3. **Developer Experience**
   - Build time: < 30s
   - Type check time: < 10s
   - Clear module boundaries

## Timeline

- **Week 1**: Phase 1 (Safe refactorings)
- **Week 2-3**: Phase 2 (api.ts split)
- **Week 4**: Phase 2 (ToolWidgets split)
- **Week 5**: Phase 3 (Pattern migrations)
- **Week 6**: Phase 4 (Type safety)
- **Week 7**: Phase 5 (Performance)
- **Week 8**: Testing & Documentation

## Next Steps

1. Review and approve this plan
2. Set up feature flags infrastructure
3. Create characterization tests for api.ts
4. Begin Phase 1 implementation
5. Daily progress reviews

## Appendix: File-by-File Changes

### api.ts → 8 services
- `project.service.ts`: 200 lines
- `session.service.ts`: 250 lines
- `claude.service.ts`: 300 lines
- `agent.service.ts`: 350 lines
- `sandbox.service.ts`: 250 lines
- `usage.service.ts`: 200 lines
- `checkpoint.service.ts`: 200 lines
- `mcp.service.ts`: 200 lines

### ToolWidgets.tsx → 20+ components
- Base components: 3 files
- Todo widgets: 2 files
- File widgets: 6 files
- Search widgets: 3 files
- Edit widgets: 5 files
- Command widgets: 3 files
- System widgets: 4 files

### Backend splits
- `agents.rs` → 4 modules
- `claude.rs` → 4 modules
- Other commands remain as-is