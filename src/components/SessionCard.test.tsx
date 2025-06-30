import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { SessionCard } from './SessionCard'
import type { AgentRun, AgentRunWithMetrics } from '@/lib/api'

// Mock framer-motion to avoid animation issues in tests
vi.mock('framer-motion', () => ({
  motion: {
    div: ({ children, ...props }: any) => <div {...props}>{children}</div>,
  },
}))

describe('SessionCard', () => {
  const mockSession: AgentRun = {
    id: 1,
    agent_id: 1,
    agent_name: 'Test Agent',
    agent_icon: '🤖',
    task: 'Test task description',
    model: 'claude-3-sonnet',
    project_path: '/test/project',
    session_id: 'test-session-123',
    status: 'running',
    pid: 12345,
    process_started_at: new Date(Date.now() - 5 * 60 * 1000).toISOString(), // 5 minutes ago
    created_at: new Date().toISOString(),
    auto_resume_enabled: false,
    resume_count: 0,
  }

  const mockSessionWithMetrics: AgentRunWithMetrics = {
    ...mockSession,
    metrics: {
      duration_ms: 300000,
      total_tokens: 1500000,
      cost_usd: 0.0456,
      message_count: 10,
    },
  }

  const defaultProps = {
    session: mockSession,
    index: 0,
    onViewOutput: vi.fn(),
    onStop: vi.fn(),
    onResume: vi.fn(),
    onRetry: vi.fn(),
    onEdit: vi.fn(),
    showStopButton: true,
    statusConfig: {
      bgColor: 'bg-blue-100',
      iconColor: 'text-blue-600',
    },
  }

  beforeEach(() => {
    vi.clearAllMocks()
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  describe('formatDuration', () => {
    it('should format duration correctly', () => {
      render(<SessionCard {...defaultProps} />)
      
      // The session started 5 minutes ago
      const durationText = screen.getByText(/5m 0s/)
      expect(durationText).toBeInTheDocument()
    })

    it('should handle duration for sessions that just started', () => {
      const justStartedSession = {
        ...mockSession,
        process_started_at: new Date().toISOString(),
      }
      render(<SessionCard {...defaultProps} session={justStartedSession} />)
      
      const durationText = screen.getByText(/0m 0s/)
      expect(durationText).toBeInTheDocument()
    })
  })

  describe('formatScheduledTime', () => {
    it('should format future scheduled time correctly', () => {
      const scheduledSession = {
        ...mockSession,
        status: 'scheduled',
        scheduled_start_time: new Date(Date.now() + 2 * 60 * 60 * 1000).toISOString(), // 2 hours from now
      }
      render(<SessionCard {...defaultProps} session={scheduledSession} />)
      
      const scheduledText = screen.getByText(/in 2h 0m/)
      expect(scheduledText).toBeInTheDocument()
    })

    it('should show "Starting soon..." for past scheduled times', () => {
      const overdueSession = {
        ...mockSession,
        status: 'scheduled',
        scheduled_start_time: new Date(Date.now() - 10 * 60 * 1000).toISOString(), // 10 minutes ago
      }
      render(<SessionCard {...defaultProps} session={overdueSession} />)
      
      const scheduledText = screen.getByText(/Starting soon.../)
      expect(scheduledText).toBeInTheDocument()
    })

    it('should format scheduled time in minutes when less than an hour', () => {
      const soonSession = {
        ...mockSession,
        status: 'scheduled',
        scheduled_start_time: new Date(Date.now() + 30 * 60 * 1000).toISOString(), // 30 minutes from now
      }
      render(<SessionCard {...defaultProps} session={soonSession} />)
      
      const scheduledText = screen.getByText(/in 30m/)
      expect(scheduledText).toBeInTheDocument()
    })
  })

  describe('formatCurrency', () => {
    it('should format currency with correct precision', () => {
      render(<SessionCard {...defaultProps} session={mockSessionWithMetrics} />)
      
      const costText = screen.getByText(/\$0.0456/)
      expect(costText).toBeInTheDocument()
    })

    it('should not show cost section when cost is zero', () => {
      const zeroMetrics = {
        ...mockSessionWithMetrics,
        metrics: { ...mockSessionWithMetrics.metrics!, cost_usd: 0 },
      }
      render(<SessionCard {...defaultProps} session={zeroMetrics} />)
      
      // When cost is 0, the cost section is not rendered
      expect(screen.queryByText('Cost')).not.toBeInTheDocument()
    })
  })

  describe('formatTokens', () => {
    it('should format millions of tokens correctly', () => {
      render(<SessionCard {...defaultProps} session={mockSessionWithMetrics} />)
      
      const tokenText = screen.getByText(/1.5M/)
      expect(tokenText).toBeInTheDocument()
    })

    it('should format thousands of tokens correctly', () => {
      const thousandsSession = {
        ...mockSessionWithMetrics,
        metrics: { ...mockSessionWithMetrics.metrics!, total_tokens: 5500 },
      }
      render(<SessionCard {...defaultProps} session={thousandsSession} />)
      
      const tokenText = screen.getByText(/5.5k/)
      expect(tokenText).toBeInTheDocument()
    })

    it('should show exact count for small numbers', () => {
      const smallSession = {
        ...mockSessionWithMetrics,
        metrics: { ...mockSessionWithMetrics.metrics!, total_tokens: 999 },
      }
      render(<SessionCard {...defaultProps} session={smallSession} />)
      
      const tokenText = screen.getByText(/999/)
      expect(tokenText).toBeInTheDocument()
    })
  })

  describe('getStatusBadge', () => {
    const testCases = [
      { status: 'running', expectedText: 'Running', expectedClasses: ['bg-green-100', 'text-green-800'] },
      { status: 'scheduled', expectedText: 'Scheduled', expectedClasses: ['bg-blue-100', 'text-blue-800'] },
      { status: 'pending', expectedText: 'Pending', expectedClasses: [] },
      { status: 'paused_usage_limit', expectedText: 'Usage Limit', expectedClasses: ['bg-orange-100', 'text-orange-800'] },
      { status: 'cancelled', expectedText: 'Cancelled', expectedClasses: ['bg-gray-100', 'text-gray-800'] },
      { status: 'failed', expectedText: 'Failed', expectedClasses: ['bg-red-100', 'text-red-800'] },
      { status: 'completed', expectedText: 'Completed', expectedClasses: ['bg-gray-100', 'text-gray-800'] },
      { status: 'unknown_status', expectedText: 'unknown_status', expectedClasses: [] },
    ]

    testCases.forEach(({ status, expectedText, expectedClasses }) => {
      it(`should render correct badge for ${status} status`, () => {
        const sessionWithStatus = { ...mockSession, status }
        render(<SessionCard {...defaultProps} session={sessionWithStatus} />)
        
        const badge = screen.getByText(expectedText)
        expect(badge).toBeInTheDocument()
        
        expectedClasses.forEach((className) => {
          expect(badge).toHaveClass(className)
        })
      })
    })
  })

  describe('canResume logic', () => {
    it('should enable Resume button when usage limit has been reset', () => {
      const pausedSession = {
        ...mockSession,
        status: 'paused_usage_limit',
        usage_limit_reset_time: new Date(Date.now() - 10 * 60 * 1000).toISOString(), // 10 minutes ago
      }
      render(<SessionCard {...defaultProps} session={pausedSession} />)
      
      const resumeButton = screen.getByRole('button', { name: /Resume/i })
      expect(resumeButton).not.toBeDisabled()
    })

    it('should disable Resume button when usage limit has not been reset', () => {
      const pausedSession = {
        ...mockSession,
        status: 'paused_usage_limit',
        usage_limit_reset_time: new Date(Date.now() + 10 * 60 * 1000).toISOString(), // 10 minutes from now
      }
      render(<SessionCard {...defaultProps} session={pausedSession} />)
      
      const resumeButton = screen.getByRole('button', { name: /Resume/i })
      expect(resumeButton).toBeDisabled()
    })

    it('should show tooltip on disabled Resume button', () => {
      const resetTime = new Date(Date.now() + 10 * 60 * 1000)
      const pausedSession = {
        ...mockSession,
        status: 'paused_usage_limit',
        usage_limit_reset_time: resetTime.toISOString(),
      }
      render(<SessionCard {...defaultProps} session={pausedSession} />)
      
      const resumeButton = screen.getByRole('button', { name: /Resume/i })
      expect(resumeButton).toHaveAttribute('title', expect.stringContaining('Usage limit resets at'))
    })
  })

  describe('Button interactions', () => {
    beforeEach(() => {
      vi.useRealTimers()
    })

    afterEach(() => {
      vi.useFakeTimers()
    })

    it('should call onViewOutput when View Output button is clicked', async () => {
      const user = userEvent.setup()
      render(<SessionCard {...defaultProps} />)
      
      const viewButton = screen.getByRole('button', { name: /View Output/i })
      await user.click(viewButton)
      
      expect(defaultProps.onViewOutput).toHaveBeenCalledWith(mockSession)
    })

    it('should call onStop when Stop button is clicked for running session', async () => {
      const user = userEvent.setup()
      render(<SessionCard {...defaultProps} />)
      
      const stopButton = screen.getByRole('button', { name: /Stop/i })
      await user.click(stopButton)
      
      expect(defaultProps.onStop).toHaveBeenCalledWith(1, 'Test Agent')
    })

    it('should call onResume when Resume button is clicked for paused session', async () => {
      const user = userEvent.setup()
      const pausedSession = {
        ...mockSession,
        status: 'paused_usage_limit',
        usage_limit_reset_time: new Date(Date.now() - 10 * 60 * 1000).toISOString(),
      }
      render(<SessionCard {...defaultProps} session={pausedSession} />)
      
      const resumeButton = screen.getByRole('button', { name: /Resume/i })
      await user.click(resumeButton)
      
      expect(defaultProps.onResume).toHaveBeenCalledWith(pausedSession)
    })

    it('should call onRetry when Retry button is clicked for failed session', async () => {
      const user = userEvent.setup()
      const failedSession = { ...mockSession, status: 'failed' }
      render(<SessionCard {...defaultProps} session={failedSession} />)
      
      const retryButton = screen.getByRole('button', { name: /Retry/i })
      await user.click(retryButton)
      
      expect(defaultProps.onRetry).toHaveBeenCalledWith(failedSession)
    })

    it('should call onEdit when Edit button is clicked for cancelled session', async () => {
      const user = userEvent.setup()
      const cancelledSession = { ...mockSession, status: 'cancelled' }
      render(<SessionCard {...defaultProps} session={cancelledSession} />)
      
      const editButton = screen.getByRole('button', { name: /Edit/i })
      await user.click(editButton)
      
      expect(defaultProps.onEdit).toHaveBeenCalledWith(cancelledSession)
    })
  })

  describe('Conditional rendering', () => {
    it('should not show Stop button when showStopButton is false', () => {
      render(<SessionCard {...defaultProps} showStopButton={false} />)
      
      expect(screen.queryByRole('button', { name: /Stop/i })).not.toBeInTheDocument()
    })

    it('should not show action buttons when handlers are not provided', () => {
      render(
        <SessionCard
          session={{ ...mockSession, status: 'failed' }}
          index={0}
          onViewOutput={vi.fn()}
          statusConfig={defaultProps.statusConfig}
        />
      )
      
      expect(screen.queryByRole('button', { name: /Retry/i })).not.toBeInTheDocument()
      expect(screen.queryByRole('button', { name: /Edit/i })).not.toBeInTheDocument()
    })

    it('should show auto-resume indicator for paused sessions with auto-resume enabled', () => {
      const autoResumeSession = {
        ...mockSession,
        status: 'paused_usage_limit',
        auto_resume_enabled: true,
      }
      render(<SessionCard {...defaultProps} session={autoResumeSession} />)
      
      expect(screen.getByText('Auto-resume enabled')).toBeInTheDocument()
    })

    it('should not show metrics section when metrics are not available', () => {
      render(<SessionCard {...defaultProps} />)
      
      expect(screen.queryByText('Cost')).not.toBeInTheDocument()
      expect(screen.queryByText('Tokens')).not.toBeInTheDocument()
    })

    it('should show PID badge when PID is available', () => {
      render(<SessionCard {...defaultProps} />)
      
      expect(screen.getByText('PID 12345')).toBeInTheDocument()
    })
  })

  describe('Accessibility', () => {
    it('should have accessible buttons with proper labels', () => {
      render(<SessionCard {...defaultProps} />)
      
      expect(screen.getByRole('button', { name: /View Output/i })).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /Stop/i })).toBeInTheDocument()
    })

    it('should have proper heading structure', () => {
      render(<SessionCard {...defaultProps} />)
      
      expect(screen.getByText('Test Agent')).toBeInTheDocument()
    })
  })
})