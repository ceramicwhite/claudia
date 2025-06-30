import { describe, it, expect, vi, beforeEach } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { api } from './api'

// Mock the invoke function
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

describe('api', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('New API methods', () => {
    describe('createScheduledAgentRun', () => {
      it('should create a scheduled agent run with correct parameters', async () => {
        const expectedRunId = 123
        vi.mocked(invoke).mockResolvedValueOnce(expectedRunId)

        const result = await api.createScheduledAgentRun(
          1,
          '/test/project',
          'Test task',
          'claude-3-sonnet',
          '2024-01-15T10:30:00Z'
        )

        expect(invoke).toHaveBeenCalledWith('create_scheduled_agent_run', {
          agentId: 1,
          projectPath: '/test/project',
          task: 'Test task',
          model: 'claude-3-sonnet',
          scheduledStartTime: '2024-01-15T10:30:00Z'
        })
        expect(result).toBe(expectedRunId)
      })

      it('should handle errors correctly', async () => {
        const error = new Error('Failed to schedule')
        vi.mocked(invoke).mockRejectedValueOnce(error)

        await expect(
          api.createScheduledAgentRun(1, '/test', 'task', 'model', '2024-01-15T10:30:00Z')
        ).rejects.toThrow('Failed to schedule')
      })
    })

    describe('getScheduledAgentRuns', () => {
      it('should retrieve scheduled agent runs', async () => {
        const mockRuns = [
          { id: 1, status: 'scheduled', agent_name: 'Agent 1' },
          { id: 2, status: 'scheduled', agent_name: 'Agent 2' }
        ]
        vi.mocked(invoke).mockResolvedValueOnce(mockRuns)

        const result = await api.getScheduledAgentRuns()

        expect(invoke).toHaveBeenCalledWith('get_scheduled_agent_runs')
        expect(result).toEqual(mockRuns)
      })

      it('should handle empty results', async () => {
        vi.mocked(invoke).mockResolvedValueOnce([])

        const result = await api.getScheduledAgentRuns()

        expect(result).toEqual([])
      })
    })

    describe('cancelScheduledAgentRun', () => {
      it('should cancel a scheduled run', async () => {
        vi.mocked(invoke).mockResolvedValueOnce(undefined)

        await api.cancelScheduledAgentRun(123)

        expect(invoke).toHaveBeenCalledWith('cancel_scheduled_agent_run', { runId: 123 })
      })

      it('should handle cancellation errors', async () => {
        vi.mocked(invoke).mockRejectedValueOnce(new Error('Run not found'))

        await expect(api.cancelScheduledAgentRun(999)).rejects.toThrow('Run not found')
      })
    })

    describe('listRunningAgentSessionsWithMetrics', () => {
      it('should list running sessions with metrics', async () => {
        const mockSessions = [
          {
            id: 1,
            agent_name: 'Test Agent',
            status: 'running',
            metrics: { total_tokens: 1000, cost_usd: 0.01 }
          }
        ]
        vi.mocked(invoke).mockResolvedValueOnce(mockSessions)

        const result = await api.listRunningAgentSessionsWithMetrics()

        expect(invoke).toHaveBeenCalledWith('list_running_sessions_with_metrics')
        expect(result).toEqual(mockSessions)
      })

      it('should handle API errors gracefully', async () => {
        vi.mocked(invoke).mockRejectedValueOnce(new Error('Database error'))

        await expect(api.listRunningAgentSessionsWithMetrics()).rejects.toThrow(
          'Failed to list running agent sessions with metrics: Database error'
        )
      })
    })

    describe('getAgentRunWithRealTimeMetrics', () => {
      it('should get agent run with real-time metrics', async () => {
        const mockRun = {
          id: 1,
          agent_name: 'Test Agent',
          metrics: { total_tokens: 5000, cost_usd: 0.05 },
          output: 'Real-time output content'
        }
        vi.mocked(invoke).mockResolvedValueOnce(mockRun)

        const result = await api.getAgentRunWithRealTimeMetrics(1)

        expect(invoke).toHaveBeenCalledWith('get_agent_run_with_real_time_metrics', { id: 1 })
        expect(result).toEqual(mockRun)
      })

      it('should handle missing runs', async () => {
        vi.mocked(invoke).mockRejectedValueOnce(new Error('Run not found'))

        await expect(api.getAgentRunWithRealTimeMetrics(999)).rejects.toThrow(
          'Failed to get agent run with real-time metrics: Run not found'
        )
      })
    })

    describe('cleanupFinishedProcesses', () => {
      it('should cleanup finished processes and return cleaned run IDs', async () => {
        const cleanedIds = [1, 2, 3]
        vi.mocked(invoke).mockResolvedValueOnce(cleanedIds)

        const result = await api.cleanupFinishedProcesses()

        expect(invoke).toHaveBeenCalledWith('cleanup_finished_processes')
        expect(result).toEqual(cleanedIds)
      })

      it('should handle cleanup errors', async () => {
        vi.mocked(invoke).mockRejectedValueOnce(new Error('Cleanup failed'))

        await expect(api.cleanupFinishedProcesses()).rejects.toThrow(
          'Failed to cleanup finished processes: Cleanup failed'
        )
      })

      it('should return empty array when no processes to clean', async () => {
        vi.mocked(invoke).mockResolvedValueOnce([])

        const result = await api.cleanupFinishedProcesses()

        expect(result).toEqual([])
      })
    })

    describe('executeAgent with autoResumeEnabled', () => {
      it('should execute agent with auto-resume enabled', async () => {
        const runId = 456
        vi.mocked(invoke).mockResolvedValueOnce(runId)

        const result = await api.executeAgent(
          1,
          '/test/project',
          'Test task',
          'claude-3-sonnet',
          true // autoResumeEnabled
        )

        expect(invoke).toHaveBeenCalledWith('execute_agent', {
          agentId: 1,
          projectPath: '/test/project',
          task: 'Test task',
          model: 'claude-3-sonnet',
          autoResumeEnabled: true
        })
        expect(result).toBe(runId)
      })

      it('should execute agent with auto-resume disabled', async () => {
        const runId = 789
        vi.mocked(invoke).mockResolvedValueOnce(runId)

        const result = await api.executeAgent(
          2,
          '/test/project',
          'Another task',
          'claude-3-opus',
          false // autoResumeEnabled
        )

        expect(invoke).toHaveBeenCalledWith('execute_agent', {
          agentId: 2,
          projectPath: '/test/project',
          task: 'Another task',
          model: 'claude-3-opus',
          autoResumeEnabled: false
        })
        expect(result).toBe(runId)
      })

      it('should handle undefined autoResumeEnabled', async () => {
        const runId = 101
        vi.mocked(invoke).mockResolvedValueOnce(runId)

        const result = await api.executeAgent(
          3,
          '/test/project',
          'Task without auto-resume',
          'claude-3-sonnet'
          // autoResumeEnabled is undefined
        )

        expect(invoke).toHaveBeenCalledWith('execute_agent', {
          agentId: 3,
          projectPath: '/test/project',
          task: 'Task without auto-resume',
          model: 'claude-3-sonnet',
          autoResumeEnabled: undefined
        })
        expect(result).toBe(101)
      })

      it('should throw descriptive error on failure', async () => {
        vi.mocked(invoke).mockRejectedValueOnce(new Error('Agent not found'))

        await expect(
          api.executeAgent(999, '/test', 'task', 'model')
        ).rejects.toThrow('Failed to execute agent: Agent not found')
      })
    })
  })

  describe('Error handling patterns', () => {
    it('should maintain consistent error message format', async () => {
      const testCases = [
        {
          method: () => api.getAgentRunWithRealTimeMetrics(1),
          expectedPrefix: 'Failed to get agent run with real-time metrics:'
        },
        {
          method: () => api.listRunningAgentSessionsWithMetrics(),
          expectedPrefix: 'Failed to list running agent sessions with metrics:'
        },
        {
          method: () => api.cleanupFinishedProcesses(),
          expectedPrefix: 'Failed to cleanup finished processes:'
        },
        {
          method: () => api.killAgentSession(1),
          expectedPrefix: 'Failed to kill agent session:'
        },
        {
          method: () => api.getSessionStatus(1),
          expectedPrefix: 'Failed to get session status:'
        }
      ]

      for (const { method, expectedPrefix } of testCases) {
        vi.mocked(invoke).mockRejectedValueOnce(new Error('Test error'))
        
        await expect(method()).rejects.toThrow(`${expectedPrefix} Test error`)
      }
    })

    it('should handle non-Error objects in catch blocks', async () => {
      vi.mocked(invoke).mockRejectedValueOnce('String error')

      await expect(api.executeAgent(1, '/test', 'task')).rejects.toThrow(
        'Failed to execute agent: Unknown error'
      )
    })
  })

  describe('Parameter validation', () => {
    it('should pass correct parameter names to invoke', async () => {
      // Test camelCase to snake_case conversion
      const testCases = [
        {
          method: () => api.createScheduledAgentRun(1, '/path', 'task', 'model', '2024-01-01T00:00:00Z'),
          expectedCommand: 'create_scheduled_agent_run',
          expectedParams: {
            agentId: 1,
            projectPath: '/path',
            task: 'task',
            model: 'model',
            scheduledStartTime: '2024-01-01T00:00:00Z'
          }
        },
        {
          method: () => api.cancelScheduledAgentRun(123),
          expectedCommand: 'cancel_scheduled_agent_run',
          expectedParams: { runId: 123 }
        },
        {
          method: () => api.getAgentRunWithRealTimeMetrics(456),
          expectedCommand: 'get_agent_run_with_real_time_metrics',
          expectedParams: { id: 456 }
        }
      ]

      for (const { method, expectedCommand, expectedParams } of testCases) {
        vi.mocked(invoke).mockResolvedValueOnce({})
        await method()
        expect(invoke).toHaveBeenCalledWith(expectedCommand, expectedParams)
      }
    })
  })

  describe('Return value handling', () => {
    it('should return default values for list methods on error', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Network error'))

      const result = await api.listAgentRuns(1)
      
      expect(result).toEqual([]) // Should return empty array instead of throwing
    })

    it('should throw for single-item methods on error', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Not found'))

      await expect(api.getAgentRun(1)).rejects.toThrow('Failed to get agent run: Not found')
    })
  })
})