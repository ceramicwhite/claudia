import { describe, it, expect, vi, beforeEach } from 'vitest';
import { SessionService } from './session.service';
import type { ProjectSession, SessionHistory } from '@/lib/api.types';

// Mock the invoke function
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

describe('SessionService', () => {
  let service: SessionService;
  const mockInvoke = vi.hoisted(() => vi.fn());

  beforeEach(() => {
    vi.resetAllMocks();
    vi.doMock('@tauri-apps/api/core', () => ({
      invoke: mockInvoke
    }));
    service = new SessionService({ enableLogging: false });
  });

  describe('getProjectSessions', () => {
    it('should get project sessions', async () => {
      const mockSessions: ProjectSession[] = [
        {
          session_id: 'session-1',
          status: 'active',
          start_time: '2024-01-01T00:00:00Z',
          last_active: '2024-01-01T01:00:00Z',
          message_count: 10,
          claude_version: '1.0.0',
          environment: 'production',
          pid: 1234
        },
        {
          session_id: 'session-2',
          status: 'inactive',
          start_time: '2024-01-01T02:00:00Z',
          last_active: '2024-01-01T03:00:00Z',
          message_count: 5,
          claude_version: '1.0.0',
          environment: 'production',
          pid: 5678
        }
      ];

      mockInvoke.mockResolvedValueOnce(mockSessions);

      const result = await service.getProjectSessions('project-123');

      expect(mockInvoke).toHaveBeenCalledWith('get_project_sessions', {
        projectPath: 'project-123'
      });
      expect(result).toEqual(mockSessions);
    });

    it('should handle errors when getting project sessions', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Directory not found'));

      await expect(service.getProjectSessions('invalid-path')).rejects.toThrow();
    });
  });

  describe('openNewSession', () => {
    it('should open a new session with all parameters', async () => {
      const sessionId = 'new-session-123';
      mockInvoke.mockResolvedValueOnce(sessionId);

      const result = await service.openNewSession(
        'project-path',
        'Hello Claude',
        'agent-123',
        true,
        'prod'
      );

      expect(mockInvoke).toHaveBeenCalledWith('open_new_session', {
        projectPath: 'project-path',
        message: 'Hello Claude',
        agentId: 'agent-123',
        resumeSession: true,
        environment: 'prod'
      });
      expect(result).toEqual(sessionId);
    });

    it('should open a new session with minimal parameters', async () => {
      const sessionId = 'new-session-456';
      mockInvoke.mockResolvedValueOnce(sessionId);

      const result = await service.openNewSession('project-path');

      expect(mockInvoke).toHaveBeenCalledWith('open_new_session', {
        projectPath: 'project-path',
        message: null,
        agentId: null,
        resumeSession: false,
        environment: null
      });
      expect(result).toEqual(sessionId);
    });
  });

  describe('loadSessionHistory', () => {
    it('should load session history', async () => {
      const mockHistory: SessionHistory = {
        session_id: 'session-123',
        messages: [
          {
            role: 'user',
            content: 'Hello',
            timestamp: '2024-01-01T00:00:00Z'
          },
          {
            role: 'assistant',
            content: 'Hi there!',
            timestamp: '2024-01-01T00:00:01Z'
          }
        ],
        metadata: {
          start_time: '2024-01-01T00:00:00Z',
          message_count: 2
        }
      };

      mockInvoke.mockResolvedValueOnce(mockHistory);

      const result = await service.loadSessionHistory('project-path', 'session-123');

      expect(mockInvoke).toHaveBeenCalledWith('load_session_history', {
        projectPath: 'project-path',
        sessionId: 'session-123'
      });
      expect(result).toEqual(mockHistory);
    });
  });

  describe('trackSessionMessages', () => {
    it('should track session messages', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await service.trackSessionMessages('session-123');

      expect(mockInvoke).toHaveBeenCalledWith('track_session_messages', {
        sessionId: 'session-123'
      });
    });

    it('should handle errors when tracking messages', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Session not found'));

      await expect(service.trackSessionMessages('invalid-session')).rejects.toThrow();
    });
  });

  describe('caching', () => {
    it('should cache getProjectSessions results', async () => {
      const mockSessions: ProjectSession[] = [{
        session_id: 'cached-1',
        status: 'active',
        start_time: '2024-01-01T00:00:00Z',
        last_active: '2024-01-01T01:00:00Z',
        message_count: 1,
        claude_version: '1.0.0',
        environment: 'production',
        pid: 9999
      }];

      mockInvoke.mockResolvedValueOnce(mockSessions);

      // First call - should hit the API
      const result1 = await service.getProjectSessionsCached('project-cached');
      expect(mockInvoke).toHaveBeenCalledTimes(1);
      expect(result1).toEqual(mockSessions);

      // Second call - should use cache
      const result2 = await service.getProjectSessionsCached('project-cached');
      expect(mockInvoke).toHaveBeenCalledTimes(1); // Still only 1 call
      expect(result2).toEqual(mockSessions);
    });
  });
});