import { describe, it, expect, vi, beforeEach } from 'vitest';
import { AgentService } from './agent.service';
import { TAURI_COMMANDS, ERROR_MESSAGES } from '@/constants';
import type { Agent } from '@/lib/api.types';

// Mock the invoke function
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

describe('AgentService', () => {
  let service: AgentService;
  const mockInvoke = vi.hoisted(() => vi.fn());

  beforeEach(() => {
    vi.resetAllMocks();
    vi.doMock('@tauri-apps/api/core', () => ({
      invoke: mockInvoke
    }));
    service = new AgentService({ enableLogging: false });
  });

  describe('listAgents', () => {
    it('should list all agents', async () => {
      const mockAgents: Agent[] = [
        {
          id: 1,
          name: 'Test Agent',
          icon: 'bot',
          system_prompt: 'Test prompt',
          default_task: 'Test task',
          model: 'sonnet',
          sandbox_enabled: true,
          enable_file_read: true,
          enable_file_write: true,
          enable_network: false,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z'
        }
      ];

      mockInvoke.mockResolvedValueOnce(mockAgents);

      const result = await service.listAgents();

      expect(mockInvoke).toHaveBeenCalledWith(TAURI_COMMANDS.LIST_AGENTS);
      expect(result).toEqual(mockAgents);
    });

    it('should handle errors when listing agents', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Network error'));

      await expect(service.listAgents()).rejects.toThrow(ERROR_MESSAGES.FAILED_TO_LIST_AGENTS);
    });
  });

  describe('createAgent', () => {
    it('should create an agent with all parameters', async () => {
      const newAgent: Agent = {
        id: 2,
        name: 'New Agent',
        icon: 'code',
        system_prompt: 'New prompt',
        default_task: 'New task',
        model: 'opus',
        sandbox_enabled: false,
        enable_file_read: true,
        enable_file_write: false,
        enable_network: true,
        created_at: '2024-01-02T00:00:00Z',
        updated_at: '2024-01-02T00:00:00Z'
      };

      mockInvoke.mockResolvedValueOnce(newAgent);

      const result = await service.createAgent(
        'New Agent',
        'code',
        'New prompt',
        'New task',
        'opus',
        false,
        true,
        false,
        true
      );

      expect(mockInvoke).toHaveBeenCalledWith(
        TAURI_COMMANDS.CREATE_AGENT,
        {
          name: 'New Agent',
          icon: 'code',
          systemPrompt: 'New prompt',
          defaultTask: 'New task',
          model: 'opus',
          sandboxEnabled: false,
          enableFileRead: true,
          enableFileWrite: false,
          enableNetwork: true
        }
      );
      expect(result).toEqual(newAgent);
    });

    it('should create an agent with minimal parameters', async () => {
      const newAgent: Agent = {
        id: 3,
        name: 'Minimal Agent',
        icon: 'bot',
        system_prompt: 'Minimal prompt',
        model: 'sonnet',
        sandbox_enabled: true,
        enable_file_read: true,
        enable_file_write: true,
        enable_network: false,
        created_at: '2024-01-03T00:00:00Z',
        updated_at: '2024-01-03T00:00:00Z'
      };

      mockInvoke.mockResolvedValueOnce(newAgent);

      const result = await service.createAgent(
        'Minimal Agent',
        'bot',
        'Minimal prompt'
      );

      expect(mockInvoke).toHaveBeenCalledWith(
        TAURI_COMMANDS.CREATE_AGENT,
        {
          name: 'Minimal Agent',
          icon: 'bot',
          systemPrompt: 'Minimal prompt',
          defaultTask: undefined,
          model: undefined,
          sandboxEnabled: undefined,
          enableFileRead: undefined,
          enableFileWrite: undefined,
          enableNetwork: undefined
        }
      );
      expect(result).toEqual(newAgent);
    });

    it('should handle errors when creating agent', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Database error'));

      await expect(
        service.createAgent('Test', 'bot', 'Test prompt')
      ).rejects.toThrow(ERROR_MESSAGES.FAILED_TO_CREATE_AGENT);
    });
  });

  describe('updateAgent', () => {
    it('should update an agent', async () => {
      const updatedAgent: Agent = {
        id: 1,
        name: 'Updated Agent',
        icon: 'robot',
        system_prompt: 'Updated prompt',
        default_task: 'Updated task',
        model: 'haiku',
        sandbox_enabled: true,
        enable_file_read: false,
        enable_file_write: false,
        enable_network: true,
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-04T00:00:00Z'
      };

      mockInvoke.mockResolvedValueOnce(updatedAgent);

      const result = await service.updateAgent(
        1,
        'Updated Agent',
        'robot',
        'Updated prompt',
        'Updated task',
        'haiku',
        true,
        false,
        false,
        true
      );

      expect(mockInvoke).toHaveBeenCalledWith(
        TAURI_COMMANDS.UPDATE_AGENT,
        {
          id: 1,
          name: 'Updated Agent',
          icon: 'robot',
          systemPrompt: 'Updated prompt',
          defaultTask: 'Updated task',
          model: 'haiku',
          sandboxEnabled: true,
          enableFileRead: false,
          enableFileWrite: false,
          enableNetwork: true
        }
      );
      expect(result).toEqual(updatedAgent);
    });
  });

  describe('deleteAgent', () => {
    it('should delete an agent', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await service.deleteAgent(1);

      expect(mockInvoke).toHaveBeenCalledWith(
        TAURI_COMMANDS.DELETE_AGENT,
        { id: 1 }
      );
    });

    it('should handle errors when deleting agent', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Not found'));

      await expect(service.deleteAgent(999)).rejects.toThrow(ERROR_MESSAGES.FAILED_TO_DELETE_AGENT);
    });
  });

  describe('executeAgent', () => {
    it('should execute an agent with task', async () => {
      const sessionId = 'session-123';
      mockInvoke.mockResolvedValueOnce(sessionId);

      const result = await service.executeAgent(1, 'Do something', 'project-path');

      expect(mockInvoke).toHaveBeenCalledWith(
        TAURI_COMMANDS.EXECUTE_AGENT,
        {
          agentId: 1,
          task: 'Do something',
          projectPath: 'project-path'
        }
      );
      expect(result).toEqual(sessionId);
    });

    it('should execute an agent without task', async () => {
      const sessionId = 'session-456';
      mockInvoke.mockResolvedValueOnce(sessionId);

      const result = await service.executeAgent(2, null, 'project-path');

      expect(mockInvoke).toHaveBeenCalledWith(
        TAURI_COMMANDS.EXECUTE_AGENT,
        {
          agentId: 2,
          task: null,
          projectPath: 'project-path'
        }
      );
      expect(result).toEqual(sessionId);
    });
  });

  describe('getAgentRun', () => {
    it('should get an agent run by id', async () => {
      const mockRun = {
        id: 1,
        agent_id: 1,
        session_id: 'session-123',
        task: 'Test task',
        status: 'completed',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T01:00:00Z'
      };

      mockInvoke.mockResolvedValueOnce(mockRun);

      const result = await service.getAgentRun(1);

      expect(mockInvoke).toHaveBeenCalledWith(
        TAURI_COMMANDS.GET_AGENT_RUN,
        { runId: 1 }
      );
      expect(result).toEqual(mockRun);
    });
  });

  describe('killAgentSession', () => {
    it('should kill an agent session', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await service.killAgentSession('session-123');

      expect(mockInvoke).toHaveBeenCalledWith(
        TAURI_COMMANDS.KILL_AGENT_SESSION,
        { sessionId: 'session-123' }
      );
    });
  });
});