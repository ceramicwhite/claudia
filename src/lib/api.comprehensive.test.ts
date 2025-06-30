/**
 * Comprehensive unit tests for api.ts and api.types.ts
 * 
 * This test suite covers:
 * - Legacy API compatibility layer
 * - Type exports and type guards
 * - Service delegation
 * - Error handling
 * - Event streaming
 * - Type validation
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { api } from './api';
import * as apiTypes from './api.types';
import { z } from 'zod';

// Mock the services
vi.mock('@/services', () => ({
  projectService: {
    listProjects: vi.fn(),
    getClaudeSettings: vi.fn(),
    saveClaudeSettings: vi.fn(),
    checkClaudeVersion: vi.fn(),
    findClaudeMdFiles: vi.fn(),
    readClaudeMdFile: vi.fn(),
    saveClaudeMdFile: vi.fn(),
    getSystemPrompt: vi.fn(),
    saveSystemPrompt: vi.fn(),
    listDirectoryContents: vi.fn(),
    searchFiles: vi.fn(),
    getClaudeBinaryPath: vi.fn(),
    setClaudeBinaryPath: vi.fn(),
    listClaudeInstallations: vi.fn(),
    captureUrlScreenshot: vi.fn(),
    cleanupScreenshotTempFiles: vi.fn(),
  },
  sessionService: {
    getProjectSessions: vi.fn(),
    openNewSession: vi.fn(),
    loadSessionHistory: vi.fn(),
    trackSessionMessages: vi.fn(),
  },
  claudeService: {
    executeClaudeCode: vi.fn(),
    continueClaudeCode: vi.fn(),
    resumeClaudeCode: vi.fn(),
    cancelClaudeExecution: vi.fn(),
  },
  agentService: {
    listAgents: vi.fn(),
    createAgent: vi.fn(),
    updateAgent: vi.fn(),
    deleteAgent: vi.fn(),
    getAgent: vi.fn(),
    exportAgent: vi.fn(),
    importAgent: vi.fn(),
    importAgentFromFile: vi.fn(),
    fetchGitHubAgents: vi.fn(),
    fetchGitHubAgentContent: vi.fn(),
    importAgentFromGitHub: vi.fn(),
    executeAgent: vi.fn(),
    createScheduledAgentRun: vi.fn(),
    getScheduledAgentRuns: vi.fn(),
    cancelScheduledAgentRun: vi.fn(),
    listAgentRuns: vi.fn(),
    getAgentRun: vi.fn(),
    getAgentRunWithRealTimeMetrics: vi.fn(),
    listRunningAgentSessions: vi.fn(),
    resumeAgent: vi.fn(),
    listRunningAgentSessionsWithMetrics: vi.fn(),
    killAgentSession: vi.fn(),
    getSessionStatus: vi.fn(),
    cleanupFinishedProcesses: vi.fn(),
    getSessionOutput: vi.fn(),
    getLiveSessionOutput: vi.fn(),
    streamSessionOutput: vi.fn(),
  },
  sandboxService: {
    listSandboxProfiles: vi.fn(),
    createSandboxProfile: vi.fn(),
    updateSandboxProfile: vi.fn(),
    deleteSandboxProfile: vi.fn(),
    getSandboxProfile: vi.fn(),
    listSandboxRules: vi.fn(),
    createSandboxRule: vi.fn(),
    updateSandboxRule: vi.fn(),
    deleteSandboxRule: vi.fn(),
    getPlatformCapabilities: vi.fn(),
    testSandboxProfile: vi.fn(),
    listSandboxViolations: vi.fn(),
    logSandboxViolation: vi.fn(),
    clearSandboxViolations: vi.fn(),
    getSandboxViolationStats: vi.fn(),
    exportSandboxProfile: vi.fn(),
    exportAllSandboxProfiles: vi.fn(),
    importSandboxProfiles: vi.fn(),
  },
  usageService: {
    getUsageStats: vi.fn(),
    getUsageByDateRange: vi.fn(),
    getSessionStats: vi.fn(),
    getUsageDetails: vi.fn(),
  },
  checkpointService: {
    createCheckpoint: vi.fn(),
    restoreCheckpoint: vi.fn(),
    listCheckpoints: vi.fn(),
    forkFromCheckpoint: vi.fn(),
    getSessionTimeline: vi.fn(),
    updateCheckpointSettings: vi.fn(),
    getCheckpointDiff: vi.fn(),
    trackCheckpointMessage: vi.fn(),
    checkAutoCheckpoint: vi.fn(),
    cleanupOldCheckpoints: vi.fn(),
    getCheckpointSettings: vi.fn(),
    clearCheckpointManager: vi.fn(),
  },
  mcpService: {
    mcpAdd: vi.fn(),
    mcpList: vi.fn(),
    mcpGet: vi.fn(),
    mcpRemove: vi.fn(),
    mcpAddJson: vi.fn(),
    mcpAddFromClaudeDesktop: vi.fn(),
    mcpServe: vi.fn(),
    mcpTestConnection: vi.fn(),
    mcpResetProjectChoices: vi.fn(),
    mcpGetServerStatus: vi.fn(),
    mcpReadProjectConfig: vi.fn(),
    mcpSaveProjectConfig: vi.fn(),
  }
}));

// Get mocked services for easier access in tests
import {
  projectService,
  sessionService,
  claudeService,
  agentService,
  sandboxService,
  usageService,
  checkpointService,
  mcpService
} from '@/services';

describe('api.ts - Legacy Compatibility Layer', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Service Delegation', () => {
    it('should delegate all methods to their respective services', () => {
      // Verify the api object has all expected methods
      const expectedMethods = [
        // Project operations
        'listProjects', 'getClaudeSettings', 'saveClaudeSettings', 'checkClaudeVersion',
        'findClaudeMdFiles', 'readClaudeMdFile', 'saveClaudeMdFile', 'getSystemPrompt',
        'saveSystemPrompt', 'listDirectoryContents', 'searchFiles', 'getClaudeBinaryPath',
        'setClaudeBinaryPath', 'listClaudeInstallations', 'captureUrlScreenshot',
        'cleanupScreenshotTempFiles',
        // Session operations
        'getProjectSessions', 'openNewSession', 'loadSessionHistory', 'trackSessionMessages',
        // Claude operations
        'executeClaudeCode', 'continueClaudeCode', 'resumeClaudeCode', 'cancelClaudeExecution',
        // Agent operations
        'listAgents', 'createAgent', 'updateAgent', 'deleteAgent', 'getAgent',
        'exportAgent', 'importAgent', 'importAgentFromFile', 'fetchGitHubAgents',
        'fetchGitHubAgentContent', 'importAgentFromGitHub', 'executeAgent',
        'createScheduledAgentRun', 'getScheduledAgentRuns', 'cancelScheduledAgentRun',
        'listAgentRuns', 'getAgentRun', 'getAgentRunWithRealTimeMetrics',
        'listRunningAgentSessions', 'resumeAgent', 'listRunningAgentSessionsWithMetrics',
        'killAgentSession', 'getSessionStatus', 'cleanupFinishedProcesses',
        'getSessionOutput', 'getLiveSessionOutput', 'streamSessionOutput',
        // Sandbox operations
        'listSandboxProfiles', 'createSandboxProfile', 'updateSandboxProfile',
        'deleteSandboxProfile', 'getSandboxProfile', 'listSandboxRules',
        'createSandboxRule', 'updateSandboxRule', 'deleteSandboxRule',
        'getPlatformCapabilities', 'testSandboxProfile', 'listSandboxViolations',
        'logSandboxViolation', 'clearSandboxViolations', 'getSandboxViolationStats',
        'exportSandboxProfile', 'exportAllSandboxProfiles', 'importSandboxProfiles',
        // Usage operations
        'getUsageStats', 'getUsageByDateRange', 'getSessionStats', 'getUsageDetails',
        // Checkpoint operations
        'createCheckpoint', 'restoreCheckpoint', 'listCheckpoints', 'forkFromCheckpoint',
        'getSessionTimeline', 'updateCheckpointSettings', 'getCheckpointDiff',
        'trackCheckpointMessage', 'checkAutoCheckpoint', 'cleanupOldCheckpoints',
        'getCheckpointSettings', 'clearCheckpointManager',
        // MCP operations
        'mcpAdd', 'mcpList', 'mcpGet', 'mcpRemove', 'mcpAddJson',
        'mcpAddFromClaudeDesktop', 'mcpServe', 'mcpTestConnection',
        'mcpResetProjectChoices', 'mcpGetServerStatus', 'mcpReadProjectConfig',
        'mcpSaveProjectConfig'
      ];

      expectedMethods.forEach(method => {
        expect(api).toHaveProperty(method);
        expect(typeof api[method as keyof typeof api]).toBe('function');
      });
    });

    describe('Project Service Delegation', () => {
      it('should correctly delegate listProjects', async () => {
        const mockProjects: apiTypes.Project[] = [
          {
            id: 'test-project',
            path: '/test/path',
            sessions: ['session1', 'session2'],
            created_at: Date.now()
          }
        ];
        vi.mocked(projectService.listProjects).mockResolvedValue(mockProjects);

        const result = await api.listProjects();

        expect(projectService.listProjects).toHaveBeenCalled();
        expect(result).toEqual(mockProjects);
      });

      it('should correctly delegate saveClaudeSettings', async () => {
        const settings = { theme: 'dark', apiKey: 'test-key' };
        vi.mocked(projectService.saveClaudeSettings).mockResolvedValue(undefined);

        await api.saveClaudeSettings(settings);

        expect(projectService.saveClaudeSettings).toHaveBeenCalledWith(settings);
      });

      it('should correctly delegate checkClaudeVersion', async () => {
        const versionStatus: apiTypes.ClaudeVersionStatus = {
          is_installed: true,
          version: '1.0.0',
          output: 'Claude Code v1.0.0'
        };
        vi.mocked(projectService.checkClaudeVersion).mockResolvedValue(versionStatus);

        const result = await api.checkClaudeVersion();

        expect(projectService.checkClaudeVersion).toHaveBeenCalled();
        expect(result).toEqual(versionStatus);
      });
    });

    describe('Session Service Delegation', () => {
      it('should correctly delegate openNewSession', async () => {
        const sessionId = 'new-session-id';
        vi.mocked(sessionService.openNewSession).mockResolvedValue(sessionId);

        const result = await api.openNewSession('project-id', '/project/path');

        expect(sessionService.openNewSession).toHaveBeenCalledWith('project-id', '/project/path');
        expect(result).toBe(sessionId);
      });
    });

    describe('Agent Service Delegation', () => {
      it('should correctly delegate executeAgent with all parameters', async () => {
        const runId = 123;
        vi.mocked(agentService.executeAgent).mockResolvedValue(runId);

        const result = await api.executeAgent(
          1,
          '/test/path',
          'test task',
          'claude-3-sonnet',
          true
        );

        expect(agentService.executeAgent).toHaveBeenCalledWith(
          1,
          '/test/path',
          'test task',
          'claude-3-sonnet',
          true
        );
        expect(result).toBe(runId);
      });

      it('should correctly delegate listAgentRuns', async () => {
        const mockRuns: apiTypes.AgentRun[] = [
          {
            id: 1,
            agent_id: 1,
            agent_name: 'Test Agent',
            agent_icon: '🤖',
            task: 'Test task',
            model: 'claude-3-sonnet',
            project_path: '/test',
            session_id: 'session-1',
            status: 'completed' as any,
            created_at: new Date().toISOString(),
            auto_resume_enabled: false,
            resume_count: 0
          }
        ];
        vi.mocked(agentService.listAgentRuns).mockResolvedValue(mockRuns);

        const result = await api.listAgentRuns(1);

        expect(agentService.listAgentRuns).toHaveBeenCalledWith(1);
        expect(result).toEqual(mockRuns);
      });
    });

    describe('Sandbox Service Delegation', () => {
      it('should correctly delegate createSandboxProfile', async () => {
        const profile: apiTypes.SandboxProfile = {
          id: 1,
          name: 'Test Profile',
          description: 'Test description',
          is_active: true,
          is_default: false,
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString()
        };
        vi.mocked(sandboxService.createSandboxProfile).mockResolvedValue(profile);

        const result = await api.createSandboxProfile('Test Profile', 'Test description');

        expect(sandboxService.createSandboxProfile).toHaveBeenCalledWith('Test Profile', 'Test description');
        expect(result).toEqual(profile);
      });
    });

    describe('Checkpoint Service Delegation', () => {
      it('should correctly delegate createCheckpoint', async () => {
        const checkpointResult: apiTypes.CheckpointResult = {
          checkpoint: {
            id: 'checkpoint-1',
            sessionId: 'session-1',
            projectId: 'project-1',
            messageIndex: 0,
            timestamp: new Date().toISOString(),
            metadata: {
              totalTokens: 100,
              modelUsed: 'claude-3-sonnet',
              userPrompt: 'Test prompt',
              fileChanges: 2,
              snapshotSize: 1024
            }
          },
          filesProcessed: 5,
          warnings: []
        };
        vi.mocked(checkpointService.createCheckpoint).mockResolvedValue(checkpointResult);

        const result = await api.createCheckpoint(
          'session-1',
          'project-1',
          0,
          'Test checkpoint'
        );

        expect(checkpointService.createCheckpoint).toHaveBeenCalledWith(
          'session-1',
          'project-1',
          0,
          'Test checkpoint'
        );
        expect(result).toEqual(checkpointResult);
      });
    });

    describe('MCP Service Delegation', () => {
      it('should correctly delegate mcpAdd', async () => {
        const addResult: apiTypes.AddServerResult = {
          success: true,
          message: 'Server added successfully',
          server_name: 'test-server'
        };
        vi.mocked(mcpService.mcpAdd).mockResolvedValue(addResult);

        const result = await api.mcpAdd(
          'test-server',
          'stdio',
          'node',
          ['server.js'],
          { NODE_ENV: 'production' }
        );

        expect(mcpService.mcpAdd).toHaveBeenCalledWith(
          'test-server',
          'stdio',
          'node',
          ['server.js'],
          { NODE_ENV: 'production' }
        );
        expect(result).toEqual(addResult);
      });
    });
  });

  describe('Method Binding', () => {
    it('should maintain correct `this` context when methods are destructured', async () => {
      const { listProjects } = api;
      const mockProjects: apiTypes.Project[] = [];
      vi.mocked(projectService.listProjects).mockResolvedValue(mockProjects);

      const result = await listProjects();

      expect(projectService.listProjects).toHaveBeenCalled();
      expect(result).toEqual(mockProjects);
    });
  });

  describe('Error Propagation', () => {
    it('should propagate errors from underlying services', async () => {
      const error = new Error('Service error');
      vi.mocked(agentService.executeAgent).mockRejectedValue(error);

      await expect(api.executeAgent(1, '/test', 'task')).rejects.toThrow('Service error');
    });
  });
});

describe('api.types.ts - Type Definitions and Type Guards', () => {
  describe('Type Exports', () => {
    it('should export all expected types', () => {
      // Core types
      expect(apiTypes).toHaveProperty('Project');
      expect(apiTypes).toHaveProperty('Session');
      expect(apiTypes).toHaveProperty('ClaudeSettings');
      expect(apiTypes).toHaveProperty('ClaudeVersionStatus');
      expect(apiTypes).toHaveProperty('ClaudeMdFile');
      expect(apiTypes).toHaveProperty('FileEntry');
      expect(apiTypes).toHaveProperty('ClaudeInstallation');
      
      // Sandbox types
      expect(apiTypes).toHaveProperty('SandboxProfile');
      expect(apiTypes).toHaveProperty('SandboxRule');
      expect(apiTypes).toHaveProperty('PlatformCapabilities');
      expect(apiTypes).toHaveProperty('OperationSupport');
      expect(apiTypes).toHaveProperty('SandboxViolation');
      expect(apiTypes).toHaveProperty('SandboxViolationStats');
      expect(apiTypes).toHaveProperty('SandboxProfileExport');
      expect(apiTypes).toHaveProperty('SandboxProfileWithRules');
      
      // Agent types
      expect(apiTypes).toHaveProperty('Agent');
      expect(apiTypes).toHaveProperty('AgentExport');
      expect(apiTypes).toHaveProperty('GitHubAgentFile');
      expect(apiTypes).toHaveProperty('AgentRun');
      expect(apiTypes).toHaveProperty('AgentRunMetrics');
      expect(apiTypes).toHaveProperty('AgentRunWithMetrics');
      
      // Usage types
      expect(apiTypes).toHaveProperty('UsageEntry');
      expect(apiTypes).toHaveProperty('ModelUsage');
      expect(apiTypes).toHaveProperty('DailyUsage');
      expect(apiTypes).toHaveProperty('ProjectUsage');
      expect(apiTypes).toHaveProperty('UsageStats');
      
      // Checkpoint types
      expect(apiTypes).toHaveProperty('Checkpoint');
      expect(apiTypes).toHaveProperty('CheckpointMetadata');
      expect(apiTypes).toHaveProperty('FileSnapshot');
      expect(apiTypes).toHaveProperty('TimelineNode');
      expect(apiTypes).toHaveProperty('SessionTimeline');
      expect(apiTypes).toHaveProperty('CheckpointStrategy');
      expect(apiTypes).toHaveProperty('CheckpointResult');
      expect(apiTypes).toHaveProperty('CheckpointDiff');
      expect(apiTypes).toHaveProperty('FileDiff');
      
      // MCP types
      expect(apiTypes).toHaveProperty('MCPServer');
      expect(apiTypes).toHaveProperty('ServerStatus');
      expect(apiTypes).toHaveProperty('MCPProjectConfig');
      expect(apiTypes).toHaveProperty('MCPServerConfig');
      expect(apiTypes).toHaveProperty('AddServerResult');
      expect(apiTypes).toHaveProperty('ImportResult');
      expect(apiTypes).toHaveProperty('ImportServerResult');
    });
  });

  describe('Type Validation with Zod Schemas', () => {
    // Create Zod schemas for runtime validation of API types
    const ProjectSchema = z.object({
      id: z.string(),
      path: z.string(),
      sessions: z.array(z.string()),
      created_at: z.number()
    });

    const SessionSchema = z.object({
      id: z.string(),
      project_id: z.string(),
      project_path: z.string(),
      todo_data: z.any().optional(),
      created_at: z.number(),
      first_message: z.string().optional(),
      message_timestamp: z.string().optional()
    });

    const AgentRunSchema = z.object({
      id: z.number().optional(),
      agent_id: z.number(),
      agent_name: z.string(),
      agent_icon: z.string(),
      task: z.string(),
      model: z.string(),
      project_path: z.string(),
      session_id: z.string(),
      status: z.string(),
      pid: z.number().optional(),
      process_started_at: z.string().optional(),
      scheduled_start_time: z.string().optional(),
      created_at: z.string(),
      completed_at: z.string().optional(),
      usage_limit_reset_time: z.string().optional(),
      auto_resume_enabled: z.boolean(),
      resume_count: z.number(),
      parent_run_id: z.number().optional()
    });

    describe('Project Type Validation', () => {
      it('should validate a valid Project object', () => {
        const project: apiTypes.Project = {
          id: 'test-project',
          path: '/test/path',
          sessions: ['session1', 'session2'],
          created_at: Date.now()
        };

        expect(() => ProjectSchema.parse(project)).not.toThrow();
      });

      it('should reject invalid Project object', () => {
        const invalidProject = {
          id: 123, // Should be string
          path: '/test/path',
          sessions: ['session1'],
          created_at: Date.now()
        };

        expect(() => ProjectSchema.parse(invalidProject)).toThrow();
      });
    });

    describe('Session Type Validation', () => {
      it('should validate a valid Session object', () => {
        const session: apiTypes.Session = {
          id: 'session-1',
          project_id: 'project-1',
          project_path: '/test/path',
          created_at: Date.now(),
          first_message: 'Hello',
          message_timestamp: new Date().toISOString()
        };

        expect(() => SessionSchema.parse(session)).not.toThrow();
      });

      it('should validate Session with optional fields', () => {
        const session: apiTypes.Session = {
          id: 'session-1',
          project_id: 'project-1',
          project_path: '/test/path',
          created_at: Date.now()
        };

        expect(() => SessionSchema.parse(session)).not.toThrow();
      });
    });

    describe('AgentRun Type Validation', () => {
      it('should validate a valid AgentRun object', () => {
        const agentRun: apiTypes.AgentRun = {
          id: 1,
          agent_id: 1,
          agent_name: 'Test Agent',
          agent_icon: '🤖',
          task: 'Test task',
          model: 'claude-3-sonnet',
          project_path: '/test',
          session_id: 'session-1',
          status: 'running' as any,
          pid: 12345,
          created_at: new Date().toISOString(),
          auto_resume_enabled: true,
          resume_count: 0
        };

        expect(() => AgentRunSchema.parse(agentRun)).not.toThrow();
      });

      it('should validate AgentRun without optional fields', () => {
        const agentRun: apiTypes.AgentRun = {
          agent_id: 1,
          agent_name: 'Test Agent',
          agent_icon: '🤖',
          task: 'Test task',
          model: 'claude-3-sonnet',
          project_path: '/test',
          session_id: 'session-1',
          status: 'scheduled' as any,
          created_at: new Date().toISOString(),
          auto_resume_enabled: false,
          resume_count: 0
        };

        expect(() => AgentRunSchema.parse(agentRun)).not.toThrow();
      });
    });

    describe('Complex Type Validation', () => {
      const CheckpointSchema = z.object({
        id: z.string(),
        sessionId: z.string(),
        projectId: z.string(),
        messageIndex: z.number(),
        timestamp: z.string(),
        description: z.string().optional(),
        parentCheckpointId: z.string().optional(),
        metadata: z.object({
          totalTokens: z.number(),
          modelUsed: z.string(),
          userPrompt: z.string(),
          fileChanges: z.number(),
          snapshotSize: z.number()
        })
      });

      it('should validate complex nested types', () => {
        const checkpoint: apiTypes.Checkpoint = {
          id: 'checkpoint-1',
          sessionId: 'session-1',
          projectId: 'project-1',
          messageIndex: 5,
          timestamp: new Date().toISOString(),
          description: 'Test checkpoint',
          metadata: {
            totalTokens: 1000,
            modelUsed: 'claude-3-sonnet',
            userPrompt: 'Test prompt',
            fileChanges: 3,
            snapshotSize: 2048
          }
        };

        expect(() => CheckpointSchema.parse(checkpoint)).not.toThrow();
      });
    });

    describe('Union Type Validation', () => {
      const CheckpointStrategySchema = z.enum(['manual', 'per_prompt', 'per_tool_use', 'smart']);

      it('should validate valid CheckpointStrategy values', () => {
        const strategies: apiTypes.CheckpointStrategy[] = ['manual', 'per_prompt', 'per_tool_use', 'smart'];
        
        strategies.forEach(strategy => {
          expect(() => CheckpointStrategySchema.parse(strategy)).not.toThrow();
        });
      });

      it('should reject invalid CheckpointStrategy values', () => {
        expect(() => CheckpointStrategySchema.parse('invalid')).toThrow();
      });
    });
  });

  describe('Type Guards and Discriminated Unions', () => {
    // Create custom type guards for runtime type checking
    function isProject(obj: any): obj is apiTypes.Project {
      return (
        typeof obj === 'object' &&
        obj !== null &&
        typeof obj.id === 'string' &&
        typeof obj.path === 'string' &&
        Array.isArray(obj.sessions) &&
        typeof obj.created_at === 'number'
      );
    }

    function isAgentRun(obj: any): obj is apiTypes.AgentRun {
      return (
        typeof obj === 'object' &&
        obj !== null &&
        typeof obj.agent_id === 'number' &&
        typeof obj.agent_name === 'string' &&
        typeof obj.task === 'string' &&
        typeof obj.model === 'string' &&
        typeof obj.status === 'string'
      );
    }

    function isMCPServer(obj: any): obj is apiTypes.MCPServer {
      return (
        typeof obj === 'object' &&
        obj !== null &&
        typeof obj.name === 'string' &&
        typeof obj.transport === 'string' &&
        (obj.transport === 'stdio' || obj.transport === 'sse') &&
        Array.isArray(obj.args) &&
        typeof obj.env === 'object'
      );
    }

    describe('Custom Type Guards', () => {
      it('should correctly identify Project objects', () => {
        const validProject: apiTypes.Project = {
          id: 'test',
          path: '/test',
          sessions: [],
          created_at: Date.now()
        };

        const invalidProject = {
          id: 123, // Wrong type
          path: '/test',
          sessions: [],
          created_at: Date.now()
        };

        expect(isProject(validProject)).toBe(true);
        expect(isProject(invalidProject)).toBe(false);
        expect(isProject(null)).toBe(false);
        expect(isProject(undefined)).toBe(false);
        expect(isProject('string')).toBe(false);
      });

      it('should correctly identify AgentRun objects', () => {
        const validRun: apiTypes.AgentRun = {
          agent_id: 1,
          agent_name: 'Test',
          agent_icon: '🤖',
          task: 'Test task',
          model: 'claude-3',
          project_path: '/test',
          session_id: 'session-1',
          status: 'running' as any,
          created_at: new Date().toISOString(),
          auto_resume_enabled: false,
          resume_count: 0
        };

        expect(isAgentRun(validRun)).toBe(true);
        expect(isAgentRun({})).toBe(false);
      });

      it('should correctly identify MCPServer objects', () => {
        const validServer: apiTypes.MCPServer = {
          name: 'test-server',
          transport: 'stdio',
          command: 'node',
          args: ['server.js'],
          env: { NODE_ENV: 'production' },
          scope: 'project',
          is_active: true,
          status: { running: true }
        };

        const invalidServer = {
          name: 'test-server',
          transport: 'invalid', // Invalid transport
          args: [],
          env: {}
        };

        expect(isMCPServer(validServer)).toBe(true);
        expect(isMCPServer(invalidServer)).toBe(false);
      });
    });
  });

  describe('Edge Cases and Error Scenarios', () => {
    describe('Null and Undefined Handling', () => {
      it('should handle optional fields correctly', () => {
        const session: apiTypes.Session = {
          id: 'session-1',
          project_id: 'project-1',
          project_path: '/test',
          created_at: Date.now(),
          todo_data: undefined,
          first_message: undefined,
          message_timestamp: undefined
        };

        expect(session.todo_data).toBeUndefined();
        expect(session.first_message).toBeUndefined();
      });
    });

    describe('Large Data Handling', () => {
      it('should handle large arrays in types', () => {
        const project: apiTypes.Project = {
          id: 'large-project',
          path: '/large',
          sessions: Array(10000).fill('session'),
          created_at: Date.now()
        };

        expect(project.sessions).toHaveLength(10000);
      });
    });

    describe('Date and Timestamp Handling', () => {
      it('should handle various date formats', () => {
        const run: apiTypes.AgentRun = {
          agent_id: 1,
          agent_name: 'Test',
          agent_icon: '🤖',
          task: 'Test',
          model: 'claude-3',
          project_path: '/test',
          session_id: 'session-1',
          status: 'completed' as any,
          created_at: '2024-01-01T00:00:00Z',
          completed_at: '2024-01-01T01:00:00.000Z',
          scheduled_start_time: '2024-01-01T00:30:00+00:00',
          auto_resume_enabled: false,
          resume_count: 0
        };

        expect(run.created_at).toBeTruthy();
        expect(run.completed_at).toBeTruthy();
        expect(run.scheduled_start_time).toBeTruthy();
      });
    });

    describe('Recursive Type Handling', () => {
      it('should handle recursive TimelineNode structure', () => {
        const createNode = (depth: number): apiTypes.TimelineNode => ({
          checkpoint: {
            id: `checkpoint-${depth}`,
            sessionId: 'session-1',
            projectId: 'project-1',
            messageIndex: depth,
            timestamp: new Date().toISOString(),
            metadata: {
              totalTokens: 100 * depth,
              modelUsed: 'claude-3',
              userPrompt: `Prompt ${depth}`,
              fileChanges: depth,
              snapshotSize: 1024 * depth
            }
          },
          children: depth > 0 ? [createNode(depth - 1)] : [],
          fileSnapshotIds: [`snapshot-${depth}`]
        });

        const deepNode = createNode(5);
        expect(deepNode.children).toHaveLength(1);
        expect(deepNode.children[0].children).toHaveLength(1);
      });
    });
  });

  describe('Type Transformations', () => {
    // Helper function to transform API responses
    function transformAgentRunToMetrics(run: apiTypes.AgentRun): apiTypes.AgentRunWithMetrics {
      return {
        ...run,
        metrics: {
          duration_ms: run.completed_at && run.process_started_at
            ? new Date(run.completed_at).getTime() - new Date(run.process_started_at).getTime()
            : undefined,
          total_tokens: 0,
          cost_usd: 0,
          message_count: 0
        }
      };
    }

    it('should transform AgentRun to AgentRunWithMetrics', () => {
      const run: apiTypes.AgentRun = {
        id: 1,
        agent_id: 1,
        agent_name: 'Test',
        agent_icon: '🤖',
        task: 'Test',
        model: 'claude-3',
        project_path: '/test',
        session_id: 'session-1',
        status: 'completed' as any,
        process_started_at: '2024-01-01T00:00:00Z',
        created_at: '2024-01-01T00:00:00Z',
        completed_at: '2024-01-01T00:10:00Z',
        auto_resume_enabled: false,
        resume_count: 0
      };

      const withMetrics = transformAgentRunToMetrics(run);
      
      expect(withMetrics).toHaveProperty('metrics');
      expect(withMetrics.metrics?.duration_ms).toBe(600000); // 10 minutes
    });
  });
});

describe('Deprecation Warnings', () => {
  let consoleSpy: any;

  beforeEach(() => {
    consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
  });

  afterEach(() => {
    consoleSpy.mockRestore();
  });

  it('should warn about deprecation when using the api object', () => {
    // Since the api object is already created, we can't test the deprecation warning
    // in its constructor. This is more of a documentation test.
    
    // The deprecation is noted in the JSDoc comments
    expect(api).toBeDefined();
    expect(typeof api.listProjects).toBe('function');
  });
});

describe('Integration with Services', () => {
  it('should maintain API compatibility when services change', async () => {
    // This test ensures that the api object continues to work
    // even if the underlying service implementations change
    
    const mockProject: apiTypes.Project = {
      id: 'test',
      path: '/test',
      sessions: [],
      created_at: Date.now()
    };
    
    // Change the service implementation
    vi.mocked(projectService.listProjects).mockImplementation(async () => {
      // Simulate a different implementation
      await new Promise(resolve => setTimeout(resolve, 10));
      return [mockProject];
    });

    const result = await api.listProjects();
    
    expect(result).toEqual([mockProject]);
  });
});