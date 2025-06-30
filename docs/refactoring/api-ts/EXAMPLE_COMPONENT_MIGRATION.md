# Example Component Migration

This file demonstrates how to migrate components from using the monolithic `api` object to using specific services.

## Example 1: Simple Component Migration

### Before
```typescript
// src/components/ProjectList.tsx
import { useState, useEffect } from 'react';
import { api } from '@/lib/api';
import type { Project } from '@/lib/api';

export function ProjectList() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const loadProjects = async () => {
      try {
        const data = await api.listProjects();
        setProjects(data);
      } catch (error) {
        console.error('Failed to load projects:', error);
      } finally {
        setLoading(false);
      }
    };
    
    loadProjects();
  }, []);

  return (
    <div>
      {projects.map(project => (
        <div key={project.id}>{project.path}</div>
      ))}
    </div>
  );
}
```

### After
```typescript
// src/components/ProjectList.tsx
import { useState, useEffect } from 'react';
import { projectService } from '@/services';
import type { Project } from '@/lib/api'; // Types still come from api

export function ProjectList() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const loadProjects = async () => {
      try {
        const data = await projectService.listProjects();
        setProjects(data);
      } catch (error) {
        console.error('Failed to load projects:', error);
      } finally {
        setLoading(false);
      }
    };
    
    loadProjects();
  }, []);

  return (
    <div>
      {projects.map(project => (
        <div key={project.id}>{project.path}</div>
      ))}
    </div>
  );
}
```

## Example 2: Component Using Multiple Services

### Before
```typescript
// src/components/AgentManager.tsx
import { api } from '@/lib/api';
import type { Agent, AgentRun, SandboxProfile } from '@/lib/api';

export function AgentManager() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [runs, setRuns] = useState<AgentRun[]>([]);
  const [profiles, setProfiles] = useState<SandboxProfile[]>([]);

  const loadData = async () => {
    const [agentList, runList, profileList] = await Promise.all([
      api.listAgents(),
      api.listAgentRuns(),
      api.listSandboxProfiles()
    ]);
    
    setAgents(agentList);
    setRuns(runList);
    setProfiles(profileList);
  };

  const executeAgent = async (agentId: number, task: string) => {
    const runId = await api.executeAgent(agentId, projectPath, task);
    await api.streamSessionOutput(runId);
  };

  const createProfile = async (name: string) => {
    const profile = await api.createSandboxProfile(name);
    await api.createSandboxRule(profile.id!, 'read', 'path', '/home', true);
  };

  // ...
}
```

### After
```typescript
// src/components/AgentManager.tsx
import { agentService, sandboxService } from '@/services';
import type { Agent, AgentRun, SandboxProfile } from '@/lib/api';

export function AgentManager() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [runs, setRuns] = useState<AgentRun[]>([]);
  const [profiles, setProfiles] = useState<SandboxProfile[]>([]);

  const loadData = async () => {
    const [agentList, runList, profileList] = await Promise.all([
      agentService.listAgents(),
      agentService.listAgentRuns(),
      sandboxService.listSandboxProfiles()
    ]);
    
    setAgents(agentList);
    setRuns(runList);
    setProfiles(profileList);
  };

  const executeAgent = async (agentId: number, task: string) => {
    const runId = await agentService.executeAgent(agentId, projectPath, task);
    await agentService.streamSessionOutput(runId);
  };

  const createProfile = async (name: string) => {
    const profile = await sandboxService.createSandboxProfile(name);
    await sandboxService.createSandboxRule(profile.id!, 'read', 'path', '/home', true);
  };

  // ...
}
```

## Example 3: Hook Migration

### Before
```typescript
// src/hooks/useClaudeSettings.ts
import { useState, useEffect } from 'react';
import { api } from '@/lib/api';
import type { ClaudeSettings } from '@/lib/api';

export function useClaudeSettings() {
  const [settings, setSettings] = useState<ClaudeSettings | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api.getClaudeSettings()
      .then(setSettings)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  const updateSettings = async (newSettings: ClaudeSettings) => {
    await api.saveClaudeSettings(newSettings);
    setSettings(newSettings);
  };

  return { settings, loading, updateSettings };
}
```

### After
```typescript
// src/hooks/useClaudeSettings.ts
import { useState, useEffect } from 'react';
import { projectService } from '@/services';
import type { ClaudeSettings } from '@/lib/api';

export function useClaudeSettings() {
  const [settings, setSettings] = useState<ClaudeSettings | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    projectService.getClaudeSettings()
      .then(setSettings)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  const updateSettings = async (newSettings: ClaudeSettings) => {
    await projectService.saveClaudeSettings(newSettings);
    setSettings(newSettings);
  };

  return { settings, loading, updateSettings };
}
```

## Benefits of Service-Based Architecture

1. **Better IntelliSense**: When you import a service, you immediately see only the methods relevant to that domain
2. **Clearer Dependencies**: It's obvious which parts of the API a component depends on
3. **Easier Testing**: You can mock individual services instead of the entire API
4. **Better Code Organization**: Related functionality is grouped together
5. **Smaller Bundle Size**: With proper tree-shaking, unused services won't be included

## Migration Tips

1. **Gradual Migration**: You don't need to migrate everything at once. The old `api` object still works.
2. **Type Imports**: Continue importing types from `@/lib/api` - they're re-exported there for convenience.
3. **Service Discovery**: Use your IDE's autocomplete to discover which service contains a method.
4. **Multiple Services**: If a component uses methods from multiple domains, import multiple services.

## Service Quick Reference

| Domain | Service | Common Methods |
|--------|---------|----------------|
| Projects & Settings | `projectService` | `listProjects`, `getClaudeSettings`, `findClaudeMdFiles` |
| Sessions | `sessionService` | `getProjectSessions`, `openNewSession`, `loadSessionHistory` |
| Claude Execution | `claudeService` | `executeClaudeCode`, `continueClaudeCode`, `cancelClaudeExecution` |
| Agents | `agentService` | `listAgents`, `createAgent`, `executeAgent`, `listAgentRuns` |
| Sandboxing | `sandboxService` | `listSandboxProfiles`, `createSandboxRule`, `getSandboxViolationStats` |
| Usage Analytics | `usageService` | `getUsageStats`, `getSessionStats`, `getUsageByDateRange` |
| Checkpoints | `checkpointService` | `createCheckpoint`, `restoreCheckpoint`, `getSessionTimeline` |
| MCP Servers | `mcpService` | `mcpList`, `mcpAdd`, `mcpTestConnection` |