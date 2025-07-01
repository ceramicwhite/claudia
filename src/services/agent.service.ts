import { BaseService, ServiceConfig } from './base.service';
import { TAURI_COMMANDS, ERROR_MESSAGES } from '@/constants';
import type { 
  Agent, 
  AgentExport,
  GitHubAgentFile,
  AgentRun,
  AgentRunWithMetrics
} from '@/lib/api.types';

/**
 * Service for agent operations
 */
export class AgentService extends BaseService {
  constructor(config?: Partial<ServiceConfig>) {
    super({
      serviceName: 'AgentService',
      enableLogging: true,
      ...config
    });
  }

  /**
   * Lists all CC agents
   * @returns Promise resolving to an array of agents
   */
  async listAgents(): Promise<Agent[]> {
    return this.invokeNoArgs<Agent[]>(
      TAURI_COMMANDS.LIST_AGENTS,
      ERROR_MESSAGES.FAILED_TO_LIST_AGENTS
    );
  }

  /**
   * Creates a new agent
   * @param name - The agent name
   * @param icon - The icon identifier
   * @param systemPrompt - The system prompt for the agent
   * @param defaultTask - Optional default task
   * @param model - Optional model (defaults to 'sonnet')
   * @param sandboxEnabled - Optional sandbox enable flag
   * @param enableFileRead - Optional file read permission
   * @param enableFileWrite - Optional file write permission
   * @param enableNetwork - Optional network permission
   * @returns Promise resolving to the created agent
   */
  async createAgent(
    name: string, 
    icon: string, 
    systemPrompt: string, 
    defaultTask?: string, 
    model?: string, 
    sandboxEnabled?: boolean,
    enableFileRead?: boolean,
    enableFileWrite?: boolean,
    enableNetwork?: boolean
  ): Promise<Agent> {
    return this.invoke<{ name: string; icon: string; systemPrompt: string; defaultTask?: string; model?: string; sandboxEnabled?: boolean; enableFileRead?: boolean; enableFileWrite?: boolean; enableNetwork?: boolean }, Agent>(
      TAURI_COMMANDS.CREATE_AGENT,
      { 
        name, 
        icon, 
        systemPrompt,
        defaultTask,
        model,
        sandboxEnabled,
        enableFileRead,
        enableFileWrite,
        enableNetwork
      },
      ERROR_MESSAGES.FAILED_TO_CREATE_AGENT
    );
  }

  /**
   * Updates an existing agent
   * @param id - The agent ID
   * @param name - The updated name
   * @param icon - The updated icon
   * @param systemPrompt - The updated system prompt
   * @param defaultTask - Optional default task
   * @param model - Optional model
   * @param sandboxEnabled - Optional sandbox enable flag
   * @param enableFileRead - Optional file read permission
   * @param enableFileWrite - Optional file write permission
   * @param enableNetwork - Optional network permission
   * @returns Promise resolving to the updated agent
   */
  async updateAgent(
    id: number, 
    name: string, 
    icon: string, 
    systemPrompt: string, 
    defaultTask?: string, 
    model?: string, 
    sandboxEnabled?: boolean,
    enableFileRead?: boolean,
    enableFileWrite?: boolean,
    enableNetwork?: boolean
  ): Promise<Agent> {
    return this.invoke<{ id: number; name: string; icon: string; systemPrompt: string; defaultTask?: string; model?: string; sandboxEnabled?: boolean; enableFileRead?: boolean; enableFileWrite?: boolean; enableNetwork?: boolean }, Agent>(
      TAURI_COMMANDS.UPDATE_AGENT,
      { 
        id, 
        name, 
        icon, 
        systemPrompt,
        defaultTask,
        model,
        sandboxEnabled,
        enableFileRead,
        enableFileWrite,
        enableNetwork
      },
      ERROR_MESSAGES.FAILED_TO_UPDATE_AGENT
    );
  }

  /**
   * Deletes an agent
   * @param id - The agent ID to delete
   * @returns Promise resolving when the agent is deleted
   */
  async deleteAgent(id: number): Promise<void> {
    return this.invoke<{ id: number }, void>(
      TAURI_COMMANDS.DELETE_AGENT,
      { id },
      ERROR_MESSAGES.FAILED_TO_DELETE_AGENT
    );
  }

  /**
   * Gets a single agent by ID
   * @param id - The agent ID
   * @returns Promise resolving to the agent
   */
  async getAgent(id: number): Promise<Agent> {
    return this.invoke<{ id: number }, Agent>(
      TAURI_COMMANDS.GET_AGENT,
      { id },
      ERROR_MESSAGES.FAILED_TO_GET_AGENT
    );
  }

  /**
   * Exports a single agent to JSON format
   * @param id - The agent ID to export
   * @returns Promise resolving to the JSON string
   */
  async exportAgent(id: number): Promise<string> {
    return this.invoke<{ id: number }, string>(
      TAURI_COMMANDS.EXPORT_AGENT,
      { id },
      ERROR_MESSAGES.FAILED_TO_EXPORT_AGENT
    );
  }

  /**
   * Imports an agent from JSON data
   * @param jsonData - The JSON string containing the agent export
   * @returns Promise resolving to the imported agent
   */
  async importAgent(jsonData: string): Promise<Agent> {
    return this.invoke<{ jsonData: string }, Agent>(
      TAURI_COMMANDS.IMPORT_AGENT,
      { jsonData },
      ERROR_MESSAGES.FAILED_TO_IMPORT_AGENT
    );
  }

  /**
   * Imports an agent from a file
   * @param filePath - The path to the JSON file
   * @returns Promise resolving to the imported agent
   */
  async importAgentFromFile(filePath: string): Promise<Agent> {
    return this.invoke<{ filePath: string }, Agent>(
      TAURI_COMMANDS.IMPORT_AGENT_FROM_FILE,
      { filePath },
      ERROR_MESSAGES.FAILED_TO_IMPORT_AGENT
    );
  }

  /**
   * Fetch list of agents from GitHub repository
   * @returns Promise resolving to list of available agents on GitHub
   */
  async fetchGitHubAgents(): Promise<GitHubAgentFile[]> {
    return this.invokeNoArgs<GitHubAgentFile[]>(TAURI_COMMANDS.FETCH_GITHUB_AGENTS, ERROR_MESSAGES.FAILED_TO_FETCH_GITHUB_AGENTS);
  }

  /**
   * Fetch and preview a specific agent from GitHub
   * @param downloadUrl - The download URL for the agent file
   * @returns Promise resolving to the agent export data
   */
  async fetchGitHubAgentContent(downloadUrl: string): Promise<AgentExport> {
    return this.invoke<{ downloadUrl: string }, AgentExport>(
      TAURI_COMMANDS.FETCH_GITHUB_AGENT_CONTENT,
      { downloadUrl },
      ERROR_MESSAGES.FAILED_TO_FETCH_GITHUB_CONTENT
    );
  }

  /**
   * Import an agent directly from GitHub
   * @param downloadUrl - The download URL for the agent file
   * @returns Promise resolving to the imported agent
   */
  async importAgentFromGitHub(downloadUrl: string): Promise<Agent> {
    return this.invoke<{ downloadUrl: string }, Agent>(
      TAURI_COMMANDS.IMPORT_AGENT_FROM_GITHUB,
      { downloadUrl },
      ERROR_MESSAGES.FAILED_TO_IMPORT_FROM_GITHUB
    );
  }

  /**
   * Executes an agent
   * @param agentId - The agent ID to execute
   * @param projectPath - The project path to run the agent in
   * @param task - The task description
   * @param model - Optional model override
   * @param autoResumeEnabled - Optional auto-resume flag
   * @returns Promise resolving to the run ID when execution starts
   */
  async executeAgent(
    agentId: number, 
    projectPath: string, 
    task: string, 
    model?: string, 
    autoResumeEnabled?: boolean
  ): Promise<number> {
    return this.invoke<{ agentId: number; projectPath: string; task: string; model?: string; autoResumeEnabled?: boolean }, number>(
      TAURI_COMMANDS.EXECUTE_AGENT,
      { agentId, projectPath, task, model, autoResumeEnabled },
      ERROR_MESSAGES.FAILED_TO_EXECUTE_AGENT
    );
  }

  /**
   * Creates a scheduled agent run
   * @param agentId - The agent ID to execute
   * @param projectPath - The project path to run the agent in
   * @param task - The task description
   * @param model - Model to use for execution
   * @param scheduledStartTime - ISO 8601 datetime string for when to execute
   * @returns Promise resolving to the scheduled run ID
   */
  async createScheduledAgentRun(
    agentId: number, 
    projectPath: string, 
    task: string, 
    model: string,
    scheduledStartTime: string
  ): Promise<number> {
    return this.invoke<{ agentId: number; projectPath: string; task: string; model: string; scheduledStartTime: string }, number>(
      TAURI_COMMANDS.CREATE_SCHEDULED_AGENT_RUN,
      { agentId, projectPath, task, model, scheduledStartTime },
      ERROR_MESSAGES.FAILED_TO_CREATE_SCHEDULED_RUN
    );
  }

  /**
   * Get a list of all scheduled agent runs
   * @returns Promise resolving to array of scheduled runs
   */
  async getScheduledAgentRuns(): Promise<AgentRunWithMetrics[]> {
    return this.invokeNoArgs<AgentRunWithMetrics[]>(TAURI_COMMANDS.GET_SCHEDULED_AGENT_RUNS, ERROR_MESSAGES.FAILED_TO_GET_SCHEDULED_RUNS);
  }

  /**
   * Cancel a scheduled agent run
   * @param runId - The run ID to cancel
   * @returns Promise resolving when schedule is cancelled
   */
  async cancelScheduledAgentRun(runId: number): Promise<void> {
    return this.invoke<{ runId: number }, void>(
      TAURI_COMMANDS.CANCEL_SCHEDULED_AGENT_RUN,
      { runId },
      ERROR_MESSAGES.FAILED_TO_CANCEL_SCHEDULED_RUN
    );
  }

  /**
   * Lists agent runs with metrics
   * @param agentId - Optional agent ID to filter runs
   * @returns Promise resolving to an array of agent runs with metrics
   */
  async listAgentRuns(agentId?: number): Promise<AgentRunWithMetrics[]> {
    // Use safeInvoke to prevent UI crashes
    return this.safeInvoke(
      this.invoke<{ agentId?: number }, AgentRunWithMetrics[]>(
        TAURI_COMMANDS.LIST_AGENT_RUNS,
        agentId ? { agentId } : {}
      ),
      []
    );
  }

  /**
   * Gets a single agent run by ID with metrics
   * @param id - The run ID
   * @returns Promise resolving to the agent run with metrics
   */
  async getAgentRun(id: number): Promise<AgentRunWithMetrics> {
    return this.invoke<{ id: number }, AgentRunWithMetrics>(
      TAURI_COMMANDS.GET_AGENT_RUN,
      { id },
      ERROR_MESSAGES.FAILED_TO_GET_AGENT_RUN
    );
  }

  /**
   * Gets a single agent run by ID with real-time metrics from JSONL
   * @param id - The run ID
   * @returns Promise resolving to the agent run with metrics
   */
  async getAgentRunWithRealTimeMetrics(id: number): Promise<AgentRunWithMetrics> {
    return this.invoke<{ id: number }, AgentRunWithMetrics>(
      TAURI_COMMANDS.GET_AGENT_RUN_WITH_REAL_TIME_METRICS,
      { id },
      ERROR_MESSAGES.FAILED_TO_GET_AGENT_RUN_WITH_METRICS
    );
  }

  /**
   * Lists all currently running agent sessions
   * @returns Promise resolving to list of running agent sessions
   */
  async listRunningAgentSessions(): Promise<AgentRun[]> {
    return this.invokeNoArgs<AgentRun[]>(TAURI_COMMANDS.LIST_RUNNING_SESSIONS, ERROR_MESSAGES.FAILED_TO_LIST_RUNNING_SESSIONS);
  }

  /**
   * Resume a paused agent session
   * @param runId - The run ID to resume
   * @returns Promise resolving to the new run ID
   */
  async resumeAgent(runId: number): Promise<number> {
    return this.invoke<{ runId: number }, number>(
      TAURI_COMMANDS.RESUME_AGENT,
      { runId },
      ERROR_MESSAGES.FAILED_TO_RESUME_AGENT
    );
  }

  /**
   * Lists all currently running agent sessions with metrics
   * @returns Promise resolving to list of running agent sessions with metrics
   */
  async listRunningAgentSessionsWithMetrics(): Promise<AgentRunWithMetrics[]> {
    return this.invokeNoArgs<AgentRunWithMetrics[]>(TAURI_COMMANDS.LIST_RUNNING_SESSIONS_WITH_METRICS, ERROR_MESSAGES.FAILED_TO_LIST_RUNNING_SESSIONS);
  }

  /**
   * Kills a running agent session
   * @param runId - The run ID to kill
   * @returns Promise resolving to whether the session was successfully killed
   */
  async killAgentSession(runId: number): Promise<boolean> {
    return this.invoke<{ runId: number }, boolean>(
      TAURI_COMMANDS.KILL_AGENT_SESSION,
      { runId },
      ERROR_MESSAGES.FAILED_TO_KILL_SESSION
    );
  }

  /**
   * Gets the status of a specific agent session
   * @param runId - The run ID to check
   * @returns Promise resolving to the session status or null if not found
   */
  async getSessionStatus(runId: number): Promise<string | null> {
    return this.invoke<{ runId: number }, string | null>(
      TAURI_COMMANDS.GET_SESSION_STATUS,
      { runId },
      ERROR_MESSAGES.FAILED_TO_GET_SESSION_STATUS
    );
  }

  /**
   * Cleanup finished processes and update their status
   * @returns Promise resolving to list of run IDs that were cleaned up
   */
  async cleanupFinishedProcesses(): Promise<number[]> {
    return this.invokeNoArgs<number[]>(TAURI_COMMANDS.CLEANUP_FINISHED_PROCESSES, ERROR_MESSAGES.FAILED_TO_CLEANUP_PROCESSES);
  }

  /**
   * Get real-time output for a running session (with live output fallback)
   * @param sessionId - The session ID to get output for
   * @returns Promise resolving to the current session output (JSONL format)
   */
  async getSessionOutput(sessionId: string): Promise<string> {
    return this.invoke<{ sessionId: string }, string>(
      TAURI_COMMANDS.GET_SESSION_OUTPUT,
      { sessionId },
      ERROR_MESSAGES.FAILED_TO_GET_SESSION_OUTPUT
    );
  }

  /**
   * Get output for an agent run from the database (for runs without session_id)
   * @param runId - The run ID to get output for
   * @returns Promise resolving to the stored JSONL output
   */
  async getAgentRunOutput(runId: number): Promise<string> {
    return this.invoke<{ runId: number }, string>(
      TAURI_COMMANDS.GET_AGENT_RUN_OUTPUT,
      { runId },
      ERROR_MESSAGES.FAILED_TO_GET_AGENT_RUN_OUTPUT
    );
  }

  /**
   * Get live output directly from process stdout buffer
   * @param runId - The run ID to get live output for
   * @returns Promise resolving to the current live output
   */
  async getLiveSessionOutput(runId: number): Promise<string> {
    return this.invoke<{ runId: number }, string>(
      TAURI_COMMANDS.GET_LIVE_SESSION_OUTPUT,
      { runId },
      ERROR_MESSAGES.FAILED_TO_GET_LIVE_OUTPUT
    );
  }

  /**
   * Start streaming real-time output for a running session
   * @param sessionId - The session ID to stream output for
   * @returns Promise that resolves when streaming starts
   */
  async streamSessionOutput(sessionId: string): Promise<void> {
    return this.invoke<{ sessionId: string }, void>(
      TAURI_COMMANDS.STREAM_SESSION_OUTPUT,
      { sessionId },
      ERROR_MESSAGES.FAILED_TO_START_STREAMING
    );
  }
}

// Export singleton instance
export const agentService = new AgentService();