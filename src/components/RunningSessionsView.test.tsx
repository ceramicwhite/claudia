import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { RunningSessionsView } from './RunningSessionsView'
import { api } from '@/lib/api'
import type { AgentRunWithMetrics } from '@/lib/api'

// Mock framer-motion
vi.mock('framer-motion', () => ({
  motion: {
    div: ({ children, ...props }: any) => <div {...props}>{children}</div>,
  },
  AnimatePresence: ({ children }: any) => children,
}))

// Mock the API
vi.mock('@/lib/api', () => ({
  api: {
    listRunningAgentSessionsWithMetrics: vi.fn(),
    cleanupFinishedProcesses: vi.fn(),
    killAgentSession: vi.fn(),
    executeAgent: vi.fn(),
  },
}))

// Mock SessionCard to simplify testing
vi.mock('./SessionCard', () => ({
  SessionCard: ({ session, onViewOutput, onStop, onResume, onRetry, onEdit }: any) => (
    <div data-testid={`session-${session.id}`}>
      <span>{session.agent_name}</span>
      <span>{session.status}</span>
      <button onClick={() => onViewOutput(session)}>View Output</button>
      {onStop && <button onClick={() => onStop(session.id, session.agent_name)}>Stop</button>}
      {onResume && <button onClick={() => onResume(session)}>Resume</button>}
      {onRetry && <button onClick={() => onRetry(session)}>Retry</button>}
      {onEdit && <button onClick={() => onEdit(session)}>Edit</button>}
    </div>
  ),
}))

// Mock SessionOutputViewer
vi.mock('./SessionOutputViewer', () => ({
  SessionOutputViewer: ({ session, onClose }: any) => (
    <div data-testid="session-output-viewer">
      <span>Output for {session.agent_name}</span>
      <button onClick={onClose}>Close</button>
    </div>
  ),
}))

describe('RunningSessionsView', () => {
  const mockSessions: AgentRunWithMetrics[] = [
    {
      id: 1,
      agent_id: 1,
      agent_name: 'Running Agent',
      agent_icon: '🏃',
      task: 'Running task',
      model: 'claude-3-sonnet',
      project_path: '/test/project',
      session_id: 'session-1',
      status: 'running',
      created_at: new Date().toISOString(),
      auto_resume_enabled: false,
      resume_count: 0,
    },
    {
      id: 2,
      agent_id: 2,
      agent_name: 'Scheduled Agent',
      agent_icon: '📅',
      task: 'Scheduled task',
      model: 'claude-3-sonnet',
      project_path: '/test/project',
      session_id: 'session-2',
      status: 'scheduled',
      scheduled_start_time: new Date(Date.now() + 60000).toISOString(),
      created_at: new Date().toISOString(),
      auto_resume_enabled: false,
      resume_count: 0,
    },
    {
      id: 3,
      agent_id: 3,
      agent_name: 'Paused Agent',
      agent_icon: '⏸️',
      task: 'Paused task',
      model: 'claude-3-sonnet',
      project_path: '/test/project',
      session_id: 'session-3',
      status: 'paused_usage_limit',
      usage_limit_reset_time: new Date(Date.now() + 3600000).toISOString(),
      created_at: new Date().toISOString(),
      auto_resume_enabled: true,
      resume_count: 1,
    },
    {
      id: 4,
      agent_id: 4,
      agent_name: 'Failed Agent',
      agent_icon: '❌',
      task: 'Failed task',
      model: 'claude-3-sonnet',
      project_path: '/test/project',
      session_id: 'session-4',
      status: 'failed',
      created_at: new Date().toISOString(),
      completed_at: new Date().toISOString(),
      auto_resume_enabled: false,
      resume_count: 0,
    },
    {
      id: 5,
      agent_id: 5,
      agent_name: 'Cancelled Agent',
      agent_icon: '🚫',
      task: 'Cancelled task',
      model: 'claude-3-sonnet',
      project_path: '/test/project',
      session_id: 'session-5',
      status: 'cancelled',
      created_at: new Date().toISOString(),
      completed_at: new Date().toISOString(),
      auto_resume_enabled: false,
      resume_count: 0,
    },
  ]

  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(api.listRunningAgentSessionsWithMetrics).mockResolvedValue(mockSessions)
    vi.mocked(api.cleanupFinishedProcesses).mockResolvedValue([])
    vi.mocked(api.killAgentSession).mockResolvedValue(true)
    vi.mocked(api.executeAgent).mockResolvedValue(6)
  })

  describe('Session grouping', () => {
    it('should group sessions by status correctly', async () => {
      render(<RunningSessionsView />)
      
      await waitFor(() => {
        expect(screen.getByText('Scheduled (1)')).toBeInTheDocument()
        expect(screen.getByText('Running (1)')).toBeInTheDocument()
        expect(screen.getByText('Paused (1)')).toBeInTheDocument()
        expect(screen.getByText('Failed (1)')).toBeInTheDocument()
        expect(screen.getByText('Cancelled (1)')).toBeInTheDocument()
      })
    })

    it('should display sessions under correct sections', async () => {
      const user = userEvent.setup()
      render(<RunningSessionsView />)
      
      await waitFor(() => {
        expect(screen.getByTestId('session-1')).toBeInTheDocument()
        expect(screen.getByTestId('session-2')).toBeInTheDocument()
        expect(screen.getByTestId('session-3')).toBeInTheDocument()
      })
      
      // Failed section is collapsed by default, expand it
      const failedSection = screen.getByText('Failed (1)')
      await user.click(failedSection)
      
      await waitFor(() => {
        expect(screen.getByTestId('session-4')).toBeInTheDocument()
      })
      
      // Cancelled section is collapsed by default, expand it
      const cancelledSection = screen.getByText('Cancelled (1)')
      await user.click(cancelledSection)
      
      await waitFor(() => {
        expect(screen.getByTestId('session-5')).toBeInTheDocument()
      })
    })
  })

  describe('Collapsible sections', () => {
    it('should toggle section visibility when clicked', async () => {
      const user = userEvent.setup()
      render(<RunningSessionsView />)
      
      await waitFor(() => {
        expect(screen.getByText('Failed (1)')).toBeInTheDocument()
      })
      
      // Failed section should be collapsed by default
      const failedSection = screen.getByText('Failed (1)')
      const failedButton = failedSection.closest('button')
      const failedChevron = failedButton?.querySelector('.lucide-chevron-down')
      expect(failedChevron).toHaveClass('-rotate-90')
      
      // Click to expand
      await user.click(failedSection)
      expect(failedChevron).not.toHaveClass('-rotate-90')
    })

    it('should have correct default expansion states', async () => {
      render(<RunningSessionsView />)
      
      await waitFor(() => {
        // Check chevron rotation for default states
        const sections = [
          { name: 'Scheduled (1)', shouldBeExpanded: true },
          { name: 'Running (1)', shouldBeExpanded: true },
          { name: 'Paused (1)', shouldBeExpanded: true },
          { name: 'Failed (1)', shouldBeExpanded: false },
          { name: 'Cancelled (1)', shouldBeExpanded: false },
        ]
        
        sections.forEach(({ name, shouldBeExpanded }) => {
          const section = screen.getByText(name)
          const button = section.closest('button')
          const chevron = button?.querySelector('.lucide-chevron-down')
          if (shouldBeExpanded) {
            expect(chevron).not.toHaveClass('-rotate-90')
          } else {
            expect(chevron).toHaveClass('-rotate-90')
          }
        })
      })
    })
  })

  describe('Empty state', () => {
    it('should show empty state when no sessions', async () => {
      vi.mocked(api.listRunningAgentSessionsWithMetrics).mockResolvedValue([])
      render(<RunningSessionsView />)
      
      await waitFor(() => {
        expect(screen.getByText('No agent sessions found')).toBeInTheDocument()
      })
    })
  })

  describe('Loading state', () => {
    it('should show loading state initially', () => {
      render(<RunningSessionsView />)
      expect(screen.getByText('Loading running sessions...')).toBeInTheDocument()
    })
  })

  describe('Refresh functionality', () => {
    it('should refresh sessions when refresh button is clicked', async () => {
      const user = userEvent.setup()
      render(<RunningSessionsView />)
      
      await waitFor(() => {
        expect(screen.getByRole('button', { name: /Refresh/i })).toBeInTheDocument()
      })
      
      // Clear mocks after initial load
      vi.clearAllMocks()
      
      const refreshButton = screen.getByRole('button', { name: /Refresh/i })
      await user.click(refreshButton)
      
      expect(api.cleanupFinishedProcesses).toHaveBeenCalled()
      expect(api.listRunningAgentSessionsWithMetrics).toHaveBeenCalled()
    })

    it('should disable refresh button while refreshing', async () => {
      const user = userEvent.setup()
      render(<RunningSessionsView />)
      
      await waitFor(() => {
        expect(screen.getByRole('button', { name: /Refresh/i })).toBeInTheDocument()
      })
      
      const refreshButton = screen.getByRole('button', { name: /Refresh/i })
      
      // Make the API call take time
      vi.mocked(api.cleanupFinishedProcesses).mockImplementation(() => 
        new Promise(resolve => setTimeout(() => resolve([]), 100))
      )
      
      await user.click(refreshButton)
      expect(refreshButton).toBeDisabled()
    })
  })

  describe('Session actions', () => {
    it('should handle Resume action', async () => {
      const user = userEvent.setup()
      render(<RunningSessionsView />)
      
      await waitFor(() => {
        expect(screen.getByTestId('session-3')).toBeInTheDocument()
      })
      
      const sessionCard = screen.getByTestId('session-3')
      const resumeButton = within(sessionCard).getByText('Resume')
      await user.click(resumeButton)
      
      expect(api.executeAgent).toHaveBeenCalledWith(
        3, // agent_id
        '/test/project',
        'Paused task',
        'claude-3-sonnet',
        true // auto_resume_enabled
      )
    })

    it('should handle Retry action', async () => {
      const user = userEvent.setup()
      render(<RunningSessionsView />)
      
      // Wait for sessions to load first
      await waitFor(() => {
        expect(screen.getByText('Failed (1)')).toBeInTheDocument()
      })
      
      // Failed section is collapsed by default, expand it first
      const failedSection = screen.getByText('Failed (1)')
      await user.click(failedSection)
      
      await waitFor(() => {
        expect(screen.getByTestId('session-4')).toBeInTheDocument()
      })
      
      const sessionCard = screen.getByTestId('session-4')
      const retryButton = within(sessionCard).getByText('Retry')
      await user.click(retryButton)
      
      expect(api.executeAgent).toHaveBeenCalledWith(
        4, // agent_id
        '/test/project',
        'Failed task',
        'claude-3-sonnet',
        false
      )
    })

    it('should handle Edit action', async () => {
      const user = userEvent.setup()
      const onEditSession = vi.fn()
      render(<RunningSessionsView onEditSession={onEditSession} />)
      
      // Wait for sessions to load first
      await waitFor(() => {
        expect(screen.getByText('Cancelled (1)')).toBeInTheDocument()
      })
      
      // Cancelled section is collapsed by default, expand it first
      const cancelledSection = screen.getByText('Cancelled (1)')
      await user.click(cancelledSection)
      
      await waitFor(() => {
        expect(screen.getByTestId('session-5')).toBeInTheDocument()
      })
      
      const sessionCard = screen.getByTestId('session-5')
      const editButton = within(sessionCard).getByText('Edit')
      await user.click(editButton)
      
      expect(onEditSession).toHaveBeenCalledWith(mockSessions[4])
    })

    it('should handle Stop action', async () => {
      const user = userEvent.setup()
      render(<RunningSessionsView />)
      
      await waitFor(() => {
        expect(screen.getByTestId('session-1')).toBeInTheDocument()
      })
      
      const sessionCard = screen.getByTestId('session-1')
      const stopButton = within(sessionCard).getByText('Stop')
      await user.click(stopButton)
      
      expect(api.killAgentSession).toHaveBeenCalledWith(1)
    })
  })

  describe('Session output viewer', () => {
    it('should open output viewer when View Output is clicked', async () => {
      const user = userEvent.setup()
      render(<RunningSessionsView />)
      
      await waitFor(() => {
        expect(screen.getByTestId('session-1')).toBeInTheDocument()
      })
      
      const sessionCard = screen.getByTestId('session-1')
      const viewButton = within(sessionCard).getByText('View Output')
      await user.click(viewButton)
      
      expect(screen.getByTestId('session-output-viewer')).toBeInTheDocument()
      expect(screen.getByText('Output for Running Agent')).toBeInTheDocument()
    })

    it('should close output viewer when close button is clicked', async () => {
      const user = userEvent.setup()
      render(<RunningSessionsView />)
      
      await waitFor(() => {
        expect(screen.getByTestId('session-1')).toBeInTheDocument()
      })
      
      const sessionCard = screen.getByTestId('session-1')
      const viewButton = within(sessionCard).getByText('View Output')
      await user.click(viewButton)
      
      const closeButton = screen.getByRole('button', { name: /Close/i })
      await user.click(closeButton)
      
      expect(screen.queryByTestId('session-output-viewer')).not.toBeInTheDocument()
    })
  })

  describe('Back button', () => {
    it('should show back button when showBackButton is true', async () => {
      const onBack = vi.fn()
      render(<RunningSessionsView showBackButton onBack={onBack} />)
      
      await waitFor(() => {
        expect(screen.getByRole('button', { name: '' })).toBeInTheDocument() // Arrow button has no text
      })
      
      const backButton = screen.getAllByRole('button')[0] // First button should be back
      await userEvent.click(backButton)
      
      expect(onBack).toHaveBeenCalled()
    })

    it('should not show back button when showBackButton is false', async () => {
      render(<RunningSessionsView />)
      
      await waitFor(() => {
        expect(screen.getByRole('button', { name: /Refresh/i })).toBeInTheDocument()
      })
      
      // Back button should not exist when showBackButton is false
      const buttons = screen.getAllByRole('button')
      // First button should be Refresh (no back button)
      expect(buttons[0]).toHaveTextContent('Refresh')
    })
  })

  describe('Auto-refresh', () => {
    beforeEach(() => {
      vi.useFakeTimers({ shouldAdvanceTime: true })
    })

    afterEach(() => {
      vi.useRealTimers()
    })

    it('should auto-refresh every 5 seconds', async () => {
      render(<RunningSessionsView />)
      
      // Wait for initial load
      await waitFor(() => {
        expect(api.listRunningAgentSessionsWithMetrics).toHaveBeenCalledTimes(1)
      })
      
      // Clear the mock to track only auto-refresh calls
      vi.clearAllMocks()
      
      // Advance timer by 5 seconds
      await vi.advanceTimersByTimeAsync(5000)
      
      await waitFor(() => {
        expect(api.listRunningAgentSessionsWithMetrics).toHaveBeenCalledTimes(1)
      })
      
      // Advance timer by another 5 seconds
      await vi.advanceTimersByTimeAsync(5000)
      
      await waitFor(() => {
        expect(api.listRunningAgentSessionsWithMetrics).toHaveBeenCalledTimes(2)
      })
    })
  })

  describe('Error handling', () => {
    it('should show error toast when loading fails', async () => {
      vi.mocked(api.listRunningAgentSessionsWithMetrics).mockRejectedValue(new Error('Failed to load'))
      render(<RunningSessionsView />)
      
      await waitFor(() => {
        expect(screen.getByText('Failed to load running sessions')).toBeInTheDocument()
      })
    })

    it('should show error toast when refresh fails', async () => {
      const user = userEvent.setup()
      render(<RunningSessionsView />)
      
      await waitFor(() => {
        expect(screen.getByRole('button', { name: /Refresh/i })).toBeInTheDocument()
      })
      
      vi.mocked(api.cleanupFinishedProcesses).mockRejectedValue(new Error('Cleanup failed'))
      
      const refreshButton = screen.getByRole('button', { name: /Refresh/i })
      await user.click(refreshButton)
      
      await waitFor(() => {
        expect(screen.getByText('Failed to refresh sessions')).toBeInTheDocument()
      })
    })
  })
})