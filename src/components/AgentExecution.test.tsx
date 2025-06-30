import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor, within, act } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { AgentExecution } from './AgentExecution'
import { ClaudeModel } from '@/constants'
import type { Agent } from '@/lib/api'
import '@/test/utils/test-setup'

// Mock framer-motion to avoid animation issues in tests
vi.mock('framer-motion', () => ({
  motion: {
    div: ({ children, ...props }: any) => <div {...props}>{children}</div>,
  },
  AnimatePresence: ({ children }: any) => children,
}))

// Mock react-virtual for virtualization
vi.mock('@tanstack/react-virtual', () => ({
  useVirtualizer: () => ({
    getVirtualItems: () => [],
    getTotalSize: () => 0,
    scrollToIndex: vi.fn(),
    measureElement: vi.fn(),
  }),
}))

// Mock services - must be inline to avoid hoisting issues
vi.mock('@/services', async () => {
  const { vi } = await import('vitest')
  
  return {
    agentService: {
      executeAgent: vi.fn(),
      createScheduledAgentRun: vi.fn(),
      listAgents: vi.fn().mockResolvedValue([]),
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
    projectService: {
      listProjects: vi.fn().mockResolvedValue([]),
      getClaudeSettings: vi.fn().mockResolvedValue({}),
      saveClaudeSettings: vi.fn().mockResolvedValue(undefined),
      checkClaudeVersion: vi.fn().mockResolvedValue({ installed: true, version: 'test' }),
      findClaudeMdFiles: vi.fn().mockResolvedValue([]),
      readClaudeMdFile: vi.fn().mockResolvedValue(''),
      saveClaudeMdFile: vi.fn().mockResolvedValue(undefined),
      listDirectoryContents: vi.fn().mockResolvedValue([]),
      searchFiles: vi.fn().mockResolvedValue([]),
      getSystemPrompt: vi.fn().mockResolvedValue(''),
      saveSystemPrompt: vi.fn().mockResolvedValue(undefined),
      getClaudeBinaryPath: vi.fn().mockResolvedValue(null),
      setClaudeBinaryPath: vi.fn().mockResolvedValue(undefined),
      createProject: vi.fn().mockResolvedValue({}),
      updateProject: vi.fn().mockResolvedValue({}),
      deleteProject: vi.fn().mockResolvedValue(undefined),
      getProjectWithStats: vi.fn().mockResolvedValue({}),
      listClaudeInstallations: vi.fn().mockResolvedValue([]),
      captureUrlScreenshot: vi.fn().mockResolvedValue(''),
      cleanupScreenshotTempFiles: vi.fn().mockResolvedValue(0),
    },
    sessionService: {
      listSessions: vi.fn().mockResolvedValue([]),
      getSession: vi.fn().mockResolvedValue(null),
      createSession: vi.fn().mockResolvedValue({}),
      updateSession: vi.fn().mockResolvedValue({}),
      getSessionStatus: vi.fn().mockResolvedValue('idle'),
      getProjectSessions: vi.fn().mockResolvedValue([]),
      openNewSession: vi.fn(),
      loadSessionHistory: vi.fn(),
      trackSessionMessages: vi.fn(),
    },
    claudeService: {
      startClaude: vi.fn().mockResolvedValue(123),
      stopClaude: vi.fn().mockResolvedValue(undefined),
      pauseClaude: vi.fn().mockResolvedValue(undefined),
      resumeClaude: vi.fn().mockResolvedValue(undefined),
      getClaudeState: vi.fn().mockResolvedValue({ status: 'idle' }),
      executeClaudeCode: vi.fn(),
      continueClaudeCode: vi.fn(),
      resumeClaudeCode: vi.fn(),
      cancelClaudeExecution: vi.fn(),
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
      getSandboxStatus: vi.fn(),
    },
    usageService: {
      getUsageSummary: vi.fn(),
      getUsageByDateRange: vi.fn(),
      updateUsageForSession: vi.fn(),
      getActiveSessionsCount: vi.fn(),
      getTotalSessions: vi.fn(),
      getUsageStats: vi.fn(),
      getSessionStats: vi.fn(),
      getUsageDetails: vi.fn(),
    },
    checkpointService: {
      listCheckpoints: vi.fn(),
      createCheckpoint: vi.fn(),
      restoreCheckpoint: vi.fn(),
      deleteCheckpoint: vi.fn(),
      forkCheckpoint: vi.fn(),
      compareCheckpoints: vi.fn(),
      mergeCheckpoints: vi.fn(),
      getCheckpointDiff: vi.fn(),
      getCheckpointTree: vi.fn(),
      forkFromCheckpoint: vi.fn(),
      getSessionTimeline: vi.fn(),
      updateCheckpointSettings: vi.fn(),
      trackCheckpointMessage: vi.fn(),
      checkAutoCheckpoint: vi.fn(),
      cleanupOldCheckpoints: vi.fn(),
      getCheckpointSettings: vi.fn(),
      clearCheckpointManager: vi.fn(),
    },
    mcpService: {
      listMcpServers: vi.fn(),
      getMcpServerStatus: vi.fn(),
      startMcpServer: vi.fn(),
      stopMcpServer: vi.fn(),
      restartMcpServer: vi.fn(),
      toggleMcpServer: vi.fn(),
      runMcpCommand: vi.fn(),
      autoStartMcpServers: vi.fn(),
      checkMcpServerConnections: vi.fn(),
      fetchMcpServersFromDesktop: vi.fn(),
      getMcpServersConfigPath: vi.fn(),
      openMcpServersConfig: vi.fn(),
      createServerFromGitHub: vi.fn(),
      updateServerFromGitHub: vi.fn(),
      checkServerHealth: vi.fn(),
      stopAllMcpServers: vi.fn(),
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
    },
  }
})

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
  emit: vi.fn(),
  once: vi.fn(),
}))

// Mock child components
vi.mock('./StreamMessage', () => ({
  StreamMessage: ({ message }: any) => (
    <div data-testid="stream-message">
      <span>{message.type}</span>
      {message.message?.content && <span>{JSON.stringify(message.message.content)}</span>}
    </div>
  ),
}))

vi.mock('./ExecutionControlBar', () => ({
  ExecutionControlBar: ({ isExecuting, onStop, totalTokens, elapsedTime }: any) => (
    <div data-testid="execution-control-bar">
      <span>Executing: {isExecuting ? 'true' : 'false'}</span>
      <span>Tokens: {totalTokens}</span>
      <span>Time: {elapsedTime}s</span>
      <button onClick={onStop}>Stop from control bar</button>
    </div>
  ),
}))

vi.mock('./ErrorBoundary', () => ({
  ErrorBoundary: ({ children }: any) => <div>{children}</div>,
}))

// Mock UI components that aren't critical for testing
vi.mock('@/components/ui/date-time-picker', () => ({
  DateTimePicker: ({ value, onChange, placeholder, disabled }: any) => (
    <input
      type="datetime-local"
      value={value || ''}
      onChange={(e) => onChange(e.target.value)}
      placeholder={placeholder}
      disabled={disabled}
      data-testid="date-time-picker"
    />
  ),
}))

vi.mock('@/components/ui/popover', () => ({
  Popover: ({ trigger, content, open, onOpenChange }: any) => (
    <div data-testid="popover">
      <div onClick={() => onOpenChange(!open)}>{trigger}</div>
      {open && <div data-testid="popover-content">{content}</div>}
    </div>
  ),
}))

// Mock AGENT_ICONS
vi.mock('./CCAgents', () => ({
  AGENT_ICONS: {
    '🤖': () => <span>🤖</span>,
    '🔧': () => <span>🔧</span>,
  },
}))

// Import mocked modules after vi.mock
import { agentService } from '@/services'
import { open } from '@tauri-apps/plugin-dialog'
import { listen } from '@tauri-apps/api/event'

// Get mocked services
const mockAgentService = vi.mocked(agentService)
const mockOpen = vi.mocked(open)
const mockListen = vi.mocked(listen)

describe('AgentExecution', () => {
  const mockAgent: Agent = {
    id: 1,
    name: 'Test Agent',
    icon: '🤖',
    system_prompt: 'Test system prompt',
    default_task: 'Default test task',
    model: ClaudeModel.SONNET,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }

  const defaultProps = {
    agent: mockAgent,
    onBack: vi.fn(),
  }

  beforeEach(() => {
    vi.clearAllMocks()
    // Don't use fake timers by default as they interfere with userEvent
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  describe('Rendering', () => {
    it('should render agent information correctly', () => {
      render(<AgentExecution {...defaultProps} />)
      
      expect(screen.getByText('Test Agent')).toBeInTheDocument()
      expect(screen.getByText('Execute CC Agent')).toBeInTheDocument()
      expect(screen.getByText('🤖')).toBeInTheDocument()
    })

    it('should render all form inputs', () => {
      render(<AgentExecution {...defaultProps} />)
      
      expect(screen.getByPlaceholderText('Select or enter project path')).toBeInTheDocument()
      expect(screen.getByText('Model')).toBeInTheDocument()
      expect(screen.getByText('Task')).toBeInTheDocument()
      expect(screen.getByText('Schedule Execution (Optional)')).toBeInTheDocument()
      expect(screen.getByLabelText('Auto-resume when Claude usage limit resets')).toBeInTheDocument()
    })

    it('should prefill form with initial values', () => {
      render(
        <AgentExecution
          {...defaultProps}
          initialTask="Initial task"
          initialModel={ClaudeModel.OPUS}
          initialProjectPath="/test/path"
        />
      )
      
      expect(screen.getByDisplayValue('Initial task')).toBeInTheDocument()
      expect(screen.getByDisplayValue('/test/path')).toBeInTheDocument()
      const opusButton = screen.getByRole('button', { name: /Claude 4 Opus/i })
      expect(opusButton).toHaveClass('border-primary')
    })

    it('should use agent default task when no initial task provided', () => {
      render(<AgentExecution {...defaultProps} />)
      
      const taskInput = screen.getByPlaceholderText('Enter the task for the agent')
      expect(taskInput).toHaveValue('Default test task')
    })

    it('should show correct model selection UI', () => {
      render(<AgentExecution {...defaultProps} />)
      
      const sonnetButton = screen.getByRole('button', { name: /Claude 4 Sonnet/i })
      const opusButton = screen.getByRole('button', { name: /Claude 4 Opus/i })
      
      expect(sonnetButton).toHaveClass('border-primary')
      expect(opusButton).not.toHaveClass('border-primary')
    })
  })

  describe('Form interactions', () => {
    it('should update project path on input', async () => {
      const user = userEvent.setup()
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.clear(pathInput)
      await user.type(pathInput, '/new/project/path')
      
      expect(pathInput).toHaveValue('/new/project/path')
    })

    it('should open directory picker when folder button is clicked', async () => {
      const user = userEvent.setup()
      mockOpen.mockResolvedValue('/selected/path')
      
      render(<AgentExecution {...defaultProps} />)
      
      // Find the folder button - it's next to the project path input
      const projectSection = screen.getByPlaceholderText('Select or enter project path').parentElement!
      const buttons = within(projectSection).getAllByRole('button')
      const folderButton = buttons[buttons.length - 1] // Last button is the folder icon
      
      await user.click(folderButton)
      
      await waitFor(() => {
        expect(open).toHaveBeenCalledWith({
          directory: true,
          multiple: false,
          title: 'Select Project Directory',
        })
      })
      
      expect(screen.getByPlaceholderText('Select or enter project path')).toHaveValue('/selected/path')
    })

    it('should handle directory picker errors', async () => {
      const user = userEvent.setup()
      mockOpen.mockRejectedValue(new Error('Permission denied'))
      
      render(<AgentExecution {...defaultProps} />)
      
      // Find the folder button next to the project path input
      const projectSection = screen.getByPlaceholderText('Select or enter project path').parentElement!
      const buttons = within(projectSection).getAllByRole('button')
      const folderButton = buttons[buttons.length - 1]
      await user.click(folderButton)
      
      await waitFor(() => {
        expect(screen.getByText(/Failed to select directory: Permission denied/)).toBeInTheDocument()
      })
    })

    it('should switch between models', async () => {
      const user = userEvent.setup()
      render(<AgentExecution {...defaultProps} />)
      
      const opusButton = screen.getByRole('button', { name: /Claude 4 Opus/i })
      await user.click(opusButton)
      
      expect(opusButton).toHaveClass('border-primary')
      expect(screen.getByRole('button', { name: /Claude 4 Sonnet/i })).not.toHaveClass('border-primary')
    })

    it('should update task input', async () => {
      const user = userEvent.setup()
      render(<AgentExecution {...defaultProps} />)
      
      const taskInput = screen.getByPlaceholderText('Enter the task for the agent')
      await user.clear(taskInput)
      await user.type(taskInput, 'New task description')
      
      expect(taskInput).toHaveValue('New task description')
    })

    it('should toggle auto-resume checkbox', async () => {
      const user = userEvent.setup()
      render(<AgentExecution {...defaultProps} />)
      
      const checkbox = screen.getByLabelText('Auto-resume when Claude usage limit resets')
      expect(checkbox).not.toBeChecked()
      
      await user.click(checkbox)
      expect(checkbox).toBeChecked()
    })

    it('should handle scheduled execution time', async () => {
      const user = userEvent.setup()
      render(<AgentExecution {...defaultProps} />)
      
      const dateTimePicker = screen.getByTestId('date-time-picker')
      const futureTime = '2025-12-31T10:00'
      
      await user.type(dateTimePicker, futureTime)
      
      expect(screen.getByText(/Agent will be queued to run at/)).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /Schedule/i })).toBeInTheDocument()
    })
  })

  describe('Execution', () => {
    it('should execute agent with correct parameters', async () => {
      const user = userEvent.setup()
      vi.mocked(agentService.executeAgent).mockResolvedValue(123)
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      await user.click(executeButton)
      
      expect(agentService.executeAgent).toHaveBeenCalledWith(
        1, // agent id
        '/test/project',
        'Default test task',
        ClaudeModel.SONNET,
        false // auto-resume
      )
    })

    it('should disable execute button when required fields are empty', () => {
      render(<AgentExecution {...defaultProps} />)
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      expect(executeButton).toBeDisabled()
    })

    it('should enable execute button when all required fields are filled', async () => {
      const user = userEvent.setup()
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      expect(executeButton).not.toBeDisabled()
    })

    it('should handle scheduled execution', async () => {
      const user = userEvent.setup()
      vi.mocked(agentService.createScheduledAgentRun).mockResolvedValue(456)
      global.alert = vi.fn()
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const dateTimePicker = screen.getByTestId('date-time-picker')
      const futureTime = '2025-12-31T10:00'
      await user.type(dateTimePicker, futureTime)
      
      const scheduleButton = screen.getByRole('button', { name: /Schedule/i })
      await user.click(scheduleButton)
      
      expect(agentService.createScheduledAgentRun).toHaveBeenCalledWith(
        1, // agent id
        '/test/project',
        'Default test task',
        ClaudeModel.SONNET,
        futureTime
      )
      
      expect(global.alert).toHaveBeenCalledWith(
        expect.stringContaining('Agent scheduled to run at')
      )
      expect(defaultProps.onBack).toHaveBeenCalled()
    })

    it('should execute on Enter key press', async () => {
      const user = userEvent.setup()
      vi.mocked(agentService.executeAgent).mockResolvedValue(123)
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const taskInput = screen.getByPlaceholderText('Enter the task for the agent')
      await user.type(taskInput, '{Enter}')
      
      expect(agentService.executeAgent).toHaveBeenCalled()
    })

    it('should show loading state during execution', async () => {
      const user = userEvent.setup()
      vi.mocked(agentService.executeAgent).mockResolvedValue(123)
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      await user.click(executeButton)
      
      expect(screen.getByText('Running')).toBeInTheDocument()
      // Find the Stop button in the task section (not the control bar)
      const taskSection = screen.getByPlaceholderText('Enter the task for the agent').parentElement!
      expect(within(taskSection).getByRole('button', { name: /Stop/i })).toBeInTheDocument()
    })

    it('should disable form inputs during execution', async () => {
      const user = userEvent.setup()
      vi.mocked(agentService.executeAgent).mockResolvedValue(123)
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path') as HTMLInputElement
      await user.type(pathInput, '/test/project')
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      await user.click(executeButton)
      
      expect(pathInput).toBeDisabled()
      expect(screen.getByPlaceholderText('Enter the task for the agent')).toBeDisabled()
      expect(screen.getByTestId('date-time-picker')).toBeDisabled()
    })
  })

  describe('Event handling', () => {
    it('should handle agent output events', async () => {
      const user = userEvent.setup()
      const mockUnlisten = vi.fn()
      let outputHandler: any = null
      
      mockListen.mockImplementation(async (event: string, handler: any) => {
        if (event === 'agent-output:123') {
          outputHandler = handler
        }
        return mockUnlisten
      })
      
      vi.mocked(agentService.executeAgent).mockResolvedValue(123)
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      await user.click(executeButton)
      
      // Wait for the handler to be set up
      await waitFor(() => {
        expect(outputHandler).not.toBeNull()
      })
      
      // Simulate receiving output
      act(() => {
        outputHandler({ payload: JSON.stringify({ type: 'assistant', message: { content: [{ type: 'text', text: 'Hello' }] } }) })
      })
      
      await waitFor(() => {
        const messages = screen.getAllByTestId('stream-message')
        expect(messages).toHaveLength(1)
        expect(messages[0]).toHaveTextContent('assistant')
      })
    })

    it('should handle agent error events', async () => {
      const user = userEvent.setup()
      const mockUnlisten = vi.fn()
      let errorHandler: any = null
      
      mockListen.mockImplementation(async (event: string, handler: any) => {
        if (event === 'agent-error:123') {
          errorHandler = handler
        }
        return mockUnlisten
      })
      
      vi.mocked(agentService.executeAgent).mockResolvedValue(123)
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      await user.click(executeButton)
      
      // Simulate receiving error
      act(() => {
        errorHandler?.({ payload: 'Agent execution failed' })
      })
      
      await waitFor(() => {
        expect(screen.getByText('Agent execution failed')).toBeInTheDocument()
      })
    })

    it('should handle agent complete events', async () => {
      const user = userEvent.setup()
      const mockUnlisten = vi.fn()
      let completeHandler: any = null
      
      mockListen.mockImplementation(async (event: string, handler: any) => {
        if (event === 'agent-complete:123') {
          completeHandler = handler
        }
        return mockUnlisten
      })
      
      vi.mocked(agentService.executeAgent).mockResolvedValue(123)
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      await user.click(executeButton)
      
      // Initially should show Stop button
      const taskSection = screen.getByPlaceholderText('Enter the task for the agent').parentElement!
      expect(within(taskSection).getByRole('button', { name: /Stop/i })).toBeInTheDocument()
      
      // Simulate successful completion
      act(() => {
        completeHandler?.({ payload: true })
      })
      
      await waitFor(() => {
        expect(within(taskSection).getByRole('button', { name: /Execute/i })).toBeInTheDocument()
        expect(within(taskSection).queryByRole('button', { name: /Stop/i })).not.toBeInTheDocument()
      })
    })

    it('should cleanup event listeners on unmount', async () => {
      const user = userEvent.setup()
      const mockUnlisten = vi.fn()
      mockListen.mockResolvedValue(mockUnlisten)
      vi.mocked(agentService.executeAgent).mockResolvedValue(123)
      
      const { unmount } = render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      await user.click(executeButton)
      
      unmount()
      
      expect(mockUnlisten).toHaveBeenCalled()
    })
  })

  describe('Navigation', () => {
    it('should call onBack when back button is clicked', async () => {
      const user = userEvent.setup()
      render(<AgentExecution {...defaultProps} />)
      
      const backButton = screen.getAllByRole('button')[0] // First button is back
      await user.click(backButton)
      
      expect(defaultProps.onBack).toHaveBeenCalled()
    })

    it('should show confirmation dialog when navigating away during execution', async () => {
      const user = userEvent.setup()
      vi.mocked(agentService.executeAgent).mockResolvedValue(123)
      window.confirm = vi.fn().mockReturnValue(false)
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      await user.click(executeButton)
      
      const backButton = screen.getAllByRole('button')[0]
      await user.click(backButton)
      
      expect(window.confirm).toHaveBeenCalledWith(
        expect.stringContaining('An agent is currently running')
      )
      expect(defaultProps.onBack).not.toHaveBeenCalled()
    })

    it('should allow navigation when user confirms', async () => {
      const user = userEvent.setup()
      vi.mocked(agentService.executeAgent).mockResolvedValue(123)
      window.confirm = vi.fn().mockReturnValue(true)
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      await user.click(executeButton)
      
      const backButton = screen.getAllByRole('button')[0]
      await user.click(backButton)
      
      expect(defaultProps.onBack).toHaveBeenCalled()
    })
  })

  describe('Output display', () => {
    it('should show empty state when no execution', () => {
      render(<AgentExecution {...defaultProps} />)
      
      expect(screen.getByText('Ready to Execute')).toBeInTheDocument()
      expect(screen.getByText('Select a project path and enter a task to run the agent')).toBeInTheDocument()
    })

    it('should show initializing state when starting execution', async () => {
      const user = userEvent.setup()
      vi.mocked(agentService.executeAgent).mockResolvedValue(123)
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      await user.click(executeButton)
      
      expect(screen.getByText('Initializing agent...')).toBeInTheDocument()
    })

    it('should update elapsed time during execution', async () => {
      const user = userEvent.setup()
      vi.mocked(agentService.executeAgent).mockResolvedValue(123)
      vi.useRealTimers()
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      await user.click(executeButton)
      
      const controlBar = screen.getByTestId('execution-control-bar')
      expect(within(controlBar).getByText('Time: 0s')).toBeInTheDocument()
      
      // Wait for timer to update
      await waitFor(() => {
        expect(within(controlBar).getByText(/Time: [1-9]\d*s/)).toBeInTheDocument()
      }, { timeout: 2000 })
      
      vi.useFakeTimers()
    })

    it('should calculate total tokens from messages', async () => {
      const user = userEvent.setup()
      const mockUnlisten = vi.fn()
      
      mockListen.mockImplementation(async (event: string, handler: any) => {
        if (event === 'agent-output:123') {
          setTimeout(() => {
            handler({ 
              payload: JSON.stringify({ 
                type: 'assistant', 
                message: { 
                  content: [{ type: 'text', text: 'Test message' }],
                  usage: { input_tokens: 100, output_tokens: 50 }
                } 
              }) 
            })
          }, 100)
        }
        return mockUnlisten
      })
      
      vi.mocked(agentService.executeAgent).mockResolvedValue(123)
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      await user.click(executeButton)
      
      await waitFor(() => {
        const controlBar = screen.getByTestId('execution-control-bar')
        expect(within(controlBar).getByText('Tokens: 150')).toBeInTheDocument()
      })
    })
  })

  describe('Copy functionality', () => {
    it('should show copy button when there are messages', async () => {
      const user = userEvent.setup()
      const mockUnlisten = vi.fn()
      let outputHandler: any = null
      
      mockListen.mockImplementation(async (event: string, handler: any) => {
        if (event === 'agent-output:123') {
          outputHandler = handler
        }
        return mockUnlisten
      })
      
      vi.mocked(agentService.executeAgent).mockResolvedValue(123)
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      await user.click(executeButton)
      
      // Wait for handler and send a message
      await waitFor(() => expect(outputHandler).not.toBeNull())
      
      act(() => {
        outputHandler({ payload: JSON.stringify({ type: 'assistant', message: { content: [{ type: 'text', text: 'Test' }] } }) })
      })
      
      await waitFor(() => {
        expect(screen.getByText('Copy Output')).toBeInTheDocument()
      })
    })

    it('should copy output as JSONL', async () => {
      const user = userEvent.setup()
      const mockUnlisten = vi.fn()
      
      // Ensure clipboard mock is set up
      vi.mocked(navigator.clipboard.writeText).mockClear()
      
      const jsonlData = JSON.stringify({ type: 'test', data: 'value' })
      
      mockListen.mockImplementation(async (event: string, handler: any) => {
        if (event === 'agent-output:123') {
          setTimeout(() => {
            handler({ payload: jsonlData })
          }, 100)
        }
        return mockUnlisten
      })
      
      vi.mocked(agentService.executeAgent).mockResolvedValue(123)
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      await user.click(executeButton)
      
      await waitFor(() => {
        expect(screen.getByText('Copy Output')).toBeInTheDocument()
      })
      
      const copyButton = screen.getByText('Copy Output')
      await user.click(copyButton)
      
      const jsonlOption = screen.getByText('Copy as JSONL')
      await user.click(jsonlOption)
      
      expect(vi.mocked(navigator.clipboard.writeText)).toHaveBeenCalledWith(jsonlData)
    })

    it('should copy output as Markdown', async () => {
      const user = userEvent.setup()
      const mockUnlisten = vi.fn()
      
      // Ensure clipboard mock is set up
      vi.mocked(navigator.clipboard.writeText).mockClear()
      
      mockListen.mockImplementation(async (event: string, handler: any) => {
        if (event === 'agent-output:123') {
          setTimeout(() => {
            handler({ 
              payload: JSON.stringify({ 
                type: 'assistant',
                message: {
                  content: [{ type: 'text', text: 'Test message' }]
                }
              })
            })
          }, 100)
        }
        return mockUnlisten
      })
      
      vi.mocked(agentService.executeAgent).mockResolvedValue(123)
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      await user.click(executeButton)
      
      await waitFor(() => {
        expect(screen.getByText('Copy Output')).toBeInTheDocument()
      })
      
      const copyButton = screen.getByText('Copy Output')
      await user.click(copyButton)
      
      const markdownOption = screen.getByText('Copy as Markdown')
      await user.click(markdownOption)
      
      expect(vi.mocked(navigator.clipboard.writeText)).toHaveBeenCalled()
      const call = vi.mocked(navigator.clipboard.writeText).mock.calls[0][0]
      expect(call).toContain('# Agent Execution: Test Agent')
      expect(call).toContain('Test message')
      expect(navigator.clipboard.writeText).toHaveBeenCalledWith(
        expect.stringContaining('Test message')
      )
    })
  })

  describe('Fullscreen modal', () => {
    it('should open fullscreen modal when button is clicked', async () => {
      const user = userEvent.setup()
      const mockUnlisten = vi.fn()
      
      mockListen.mockImplementation(async (event: string, handler: any) => {
        if (event === 'agent-output:123') {
          setTimeout(() => {
            handler({ payload: JSON.stringify({ type: 'assistant', message: { content: [{ type: 'text', text: 'Test' }] } }) })
          }, 100)
        }
        return mockUnlisten
      })
      
      vi.mocked(agentService.executeAgent).mockResolvedValue(123)
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      await user.click(executeButton)
      
      await waitFor(() => {
        expect(screen.getByText('Fullscreen')).toBeInTheDocument()
      })
      
      const fullscreenButton = screen.getByText('Fullscreen')
      await user.click(fullscreenButton)
      
      expect(screen.getByText('Test Agent - Output')).toBeInTheDocument()
    })

    it('should close fullscreen modal when close button is clicked', async () => {
      const user = userEvent.setup()
      const mockUnlisten = vi.fn()
      
      mockListen.mockImplementation(async (event: string, handler: any) => {
        if (event === 'agent-output:123') {
          setTimeout(() => {
            handler({ payload: JSON.stringify({ type: 'assistant', message: { content: [{ type: 'text', text: 'Test' }] } }) })
          }, 100)
        }
        return mockUnlisten
      })
      
      vi.mocked(agentService.executeAgent).mockResolvedValue(123)
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      await user.click(executeButton)
      
      await waitFor(() => {
        expect(screen.getByText('Fullscreen')).toBeInTheDocument()
      })
      
      const fullscreenButton = screen.getByText('Fullscreen')
      await user.click(fullscreenButton)
      
      const closeButton = screen.getByRole('button', { name: /Close/i })
      await user.click(closeButton)
      
      expect(screen.queryByText('Test Agent - Output')).not.toBeInTheDocument()
    })
  })

  describe('Error handling', () => {
    it('should display execution errors', async () => {
      const user = userEvent.setup()
      vi.mocked(agentService.executeAgent).mockRejectedValue(new Error('Execution failed'))
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      await user.click(executeButton)
      
      await waitFor(() => {
        expect(screen.getByText('Failed to execute agent')).toBeInTheDocument()
      })
    })

    it('should display scheduled execution errors', async () => {
      const user = userEvent.setup()
      vi.mocked(agentService.createScheduledAgentRun).mockRejectedValue(new Error('Scheduling failed'))
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const dateTimePicker = screen.getByTestId('date-time-picker')
      await user.type(dateTimePicker, '2025-12-31T10:00')
      
      const scheduleButton = screen.getByRole('button', { name: /Schedule/i })
      await user.click(scheduleButton)
      
      await waitFor(() => {
        expect(screen.getByText('Failed to schedule agent execution')).toBeInTheDocument()
      })
    })

    it('should clear error when selecting a new directory', async () => {
      const user = userEvent.setup()
      
      // First, trigger an error
      mockOpen.mockRejectedValueOnce(new Error('Permission denied'))
      
      render(<AgentExecution {...defaultProps} />)
      
      const folderButton = screen.getByRole('button', { name: '' })
      await user.click(folderButton)
      
      await waitFor(() => {
        expect(screen.getByText(/Failed to select directory/)).toBeInTheDocument()
      })
      
      // Then successfully select a directory
      mockOpen.mockResolvedValueOnce('/new/path')
      await user.click(folderButton)
      
      await waitFor(() => {
        expect(screen.queryByText(/Failed to select directory/)).not.toBeInTheDocument()
      })
    })
  })

  describe('Accessibility', () => {
    it('should have proper ARIA labels', () => {
      render(<AgentExecution {...defaultProps} />)
      
      expect(screen.getByPlaceholderText('Select or enter project path')).toBeInTheDocument()
      expect(screen.getByText('Model')).toBeInTheDocument()
      expect(screen.getByPlaceholderText('Enter the task for the agent')).toBeInTheDocument()
      expect(screen.getByText('Schedule Execution (Optional)')).toBeInTheDocument()
      expect(screen.getByLabelText('Auto-resume when Claude usage limit resets')).toBeInTheDocument()
    })

    it('should have proper heading hierarchy', () => {
      render(<AgentExecution {...defaultProps} />)
      
      const heading = screen.getByText('Test Agent')
      expect(heading.tagName).toBe('H2')
    })

    it('should manage focus properly during execution', async () => {
      const user = userEvent.setup()
      vi.mocked(agentService.executeAgent).mockResolvedValue(123)
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      await user.click(executeButton)
      
      // Stop button should be focusable during execution
      const stopButton = screen.getByRole('button', { name: /Stop/i })
      expect(stopButton).not.toBeDisabled()
    })
  })

  describe('Edge cases', () => {
    it('should handle empty messages array', () => {
      render(<AgentExecution {...defaultProps} />)
      
      expect(screen.queryByText('Copy Output')).not.toBeInTheDocument()
      expect(screen.queryByText('Fullscreen')).not.toBeInTheDocument()
    })

    it('should filter out empty user messages', async () => {
      const user = userEvent.setup()
      const mockUnlisten = vi.fn()
      
      mockListen.mockImplementation(async (event: string, handler: any) => {
        if (event === 'agent-output:123') {
          setTimeout(() => {
            // Send empty user message
            handler({ 
              payload: JSON.stringify({ 
                type: 'user',
                message: { content: [] }
              })
            })
            // Send valid assistant message
            handler({ 
              payload: JSON.stringify({ 
                type: 'assistant',
                message: { content: [{ type: 'text', text: 'Valid message' }] }
              })
            })
          }, 100)
        }
        return mockUnlisten
      })
      
      vi.mocked(agentService.executeAgent).mockResolvedValue(123)
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      await user.click(executeButton)
      
      await waitFor(() => {
        const messages = screen.getAllByTestId('stream-message')
        expect(messages).toHaveLength(1) // Only the valid assistant message
        expect(screen.getByText('assistant')).toBeInTheDocument()
        expect(screen.queryByText('user')).not.toBeInTheDocument()
      })
    })

    it('should handle malformed JSON in output events', async () => {
      const user = userEvent.setup()
      const mockUnlisten = vi.fn()
      
      mockListen.mockImplementation(async (event: string, handler: any) => {
        if (event === 'agent-output:123') {
          setTimeout(() => {
            handler({ payload: 'invalid json' })
          }, 100)
        }
        return mockUnlisten
      })
      
      vi.mocked(agentService.executeAgent).mockResolvedValue(123)
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      await user.click(executeButton)
      
      // Should not crash, just ignore malformed messages
      await waitFor(() => {
        expect(screen.getByText('Initializing agent...')).toBeInTheDocument()
      })
    })

    it('should handle stop functionality', async () => {
      const user = userEvent.setup()
      vi.mocked(agentService.executeAgent).mockResolvedValue(123)
      
      render(<AgentExecution {...defaultProps} />)
      
      const pathInput = screen.getByPlaceholderText('Select or enter project path')
      await user.type(pathInput, '/test/project')
      
      const executeButton = screen.getByRole('button', { name: /Execute/i })
      await user.click(executeButton)
      
      const stopButton = screen.getByRole('button', { name: /Stop/i })
      await user.click(stopButton)
      
      expect(screen.getByRole('button', { name: /Execute/i })).toBeInTheDocument()
      
      // Verify stop message was added
      await waitFor(() => {
        const messages = screen.getAllByTestId('stream-message')
        const lastMessage = messages[messages.length - 1]
        expect(within(lastMessage).getByText('result')).toBeInTheDocument()
      })
    })
  })
})