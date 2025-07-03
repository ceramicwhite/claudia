import { invoke } from "@tauri-apps/api/core";
import type { 
  Agent, 
  AgentExport, 
  GitHubAgentFile, 
  AgentRun, 
  AgentRunWithMetrics 
} from "../types";

/**
 * Agents API module
 */
export const agentsApi = {
  /**
   * Fetch list of agents from GitHub repository
   * @returns Promise resolving to list of available agents on GitHub
   */
  async fetchGitHubAgents(): Promise<GitHubAgentFile[]> {
    try {
      return await invoke<GitHubAgentFile[]>('fetch_github_agents');
    } catch (error) {
      console.error("Failed to fetch GitHub agents:", error);
      throw error;
    }
  },

  /**
   * Fetch and preview a specific agent from GitHub
   * @param downloadUrl - The download URL for the agent file
   * @returns Promise resolving to the agent export data
   */
  async fetchGitHubAgentContent(downloadUrl: string): Promise<AgentExport> {
    try {
      return await invoke<AgentExport>('fetch_github_agent_content', { downloadUrl });
    } catch (error) {
      console.error("Failed to fetch GitHub agent content:", error);
      throw error;
    }
  },

  /**
   * Import an agent directly from GitHub
   * @param downloadUrl - The download URL for the agent file
   * @returns Promise resolving to the imported agent
   */
  async importAgentFromGitHub(downloadUrl: string): Promise<Agent> {
    try {
      return await invoke<Agent>('import_agent_from_github', { downloadUrl });
    } catch (error) {
      console.error("Failed to import agent from GitHub:", error);
      throw error;
    }
  },

  /**
   * Lists all CC agents
   * @returns Promise resolving to an array of agents
   */
  async listAgents(): Promise<Agent[]> {
    try {
      return await invoke<Agent[]>('list_agents');
    } catch (error) {
      console.error("Failed to list agents:", error);
      throw error;
    }
  },

  /**
   * Creates a new agent
   * @param name - The agent name
   * @param icon - The icon identifier
   * @param system_prompt - The system prompt for the agent
   * @param default_task - Optional default task
   * @param model - Optional model (defaults to 'sonnet')
   * @param sandbox_enabled - Optional sandbox enable flag
   * @param enable_file_read - Optional file read permission
   * @param enable_file_write - Optional file write permission
   * @param enable_network - Optional network permission
   * @returns Promise resolving to the created agent
   */
  async createAgent(
    name: string, 
    icon: string, 
    system_prompt: string, 
    default_task?: string, 
    model?: string, 
    sandbox_enabled?: boolean,
    enable_file_read?: boolean,
    enable_file_write?: boolean,
    enable_network?: boolean
  ): Promise<Agent> {
    try {
      return await invoke<Agent>('create_agent', { 
        name, 
        icon, 
        systemPrompt: system_prompt,
        defaultTask: default_task,
        model,
        sandboxEnabled: sandbox_enabled,
        enableFileRead: enable_file_read,
        enableFileWrite: enable_file_write,
        enableNetwork: enable_network
      });
    } catch (error) {
      console.error("Failed to create agent:", error);
      throw error;
    }
  },

  /**
   * Updates an existing agent
   * @param id - The agent ID
   * @param name - The updated name
   * @param icon - The updated icon
   * @param system_prompt - The updated system prompt
   * @param default_task - Optional default task
   * @param model - Optional model
   * @param sandbox_enabled - Optional sandbox enable flag
   * @param enable_file_read - Optional file read permission
   * @param enable_file_write - Optional file write permission
   * @param enable_network - Optional network permission
   * @returns Promise resolving to the updated agent
   */
  async updateAgent(
    id: number, 
    name: string, 
    icon: string, 
    system_prompt: string, 
    default_task?: string, 
    model?: string, 
    sandbox_enabled?: boolean,
    enable_file_read?: boolean,
    enable_file_write?: boolean,
    enable_network?: boolean
  ): Promise<Agent> {
    try {
      return await invoke<Agent>('update_agent', { 
        id, 
        name, 
        icon, 
        systemPrompt: system_prompt,
        defaultTask: default_task,
        model,
        sandboxEnabled: sandbox_enabled,
        enableFileRead: enable_file_read,
        enableFileWrite: enable_file_write,
        enableNetwork: enable_network
      });
    } catch (error) {
      console.error("Failed to update agent:", error);
      throw error;
    }
  },

  /**
   * Deletes an agent
   * @param id - The agent ID to delete
   * @returns Promise resolving when the agent is deleted
   */
  async deleteAgent(id: number): Promise<void> {
    try {
      return await invoke('delete_agent', { id });
    } catch (error) {
      console.error("Failed to delete agent:", error);
      throw error;
    }
  },

  /**
   * Gets a single agent by ID
   * @param id - The agent ID
   * @returns Promise resolving to the agent
   */
  async getAgent(id: number): Promise<Agent> {
    try {
      return await invoke<Agent>('get_agent', { id });
    } catch (error) {
      console.error("Failed to get agent:", error);
      throw error;
    }
  },

  /**
   * Exports a single agent to JSON format
   * @param id - The agent ID to export
   * @returns Promise resolving to the JSON string
   */
  async exportAgent(id: number): Promise<string> {
    try {
      return await invoke<string>('export_agent', { id });
    } catch (error) {
      console.error("Failed to export agent:", error);
      throw error;
    }
  },

  /**
   * Imports an agent from JSON data
   * @param jsonData - The JSON string containing the agent export
   * @returns Promise resolving to the imported agent
   */
  async importAgent(jsonData: string): Promise<Agent> {
    try {
      return await invoke<Agent>('import_agent', { jsonData });
    } catch (error) {
      console.error("Failed to import agent:", error);
      throw error;
    }
  },

  /**
   * Imports an agent from a file
   * @param filePath - The path to the JSON file
   * @returns Promise resolving to the imported agent
   */
  async importAgentFromFile(filePath: string): Promise<Agent> {
    try {
      return await invoke<Agent>('import_agent_from_file', { filePath });
    } catch (error) {
      console.error("Failed to import agent from file:", error);
      throw error;
    }
  },

  /**
   * Executes an agent
   * @param agentId - The agent ID to execute
   * @param projectPath - The project path to run the agent in
   * @param task - The task description
   * @param model - Optional model override
   * @returns Promise resolving to the run ID when execution starts
   */
  async executeAgent(agentId: number, projectPath: string, task: string, model?: string, autoResumeEnabled?: boolean): Promise<number> {
    try {
      return await invoke<number>('execute_agent', { agentId, projectPath, task, model, autoResumeEnabled });
    } catch (error) {
      console.error("Failed to execute agent:", error);
      // Return a sentinel value to indicate error
      throw new Error(`Failed to execute agent: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  },

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
    try {
      return await invoke<number>('create_scheduled_agent_run', { 
        agentId, 
        projectPath, 
        task, 
        model,
        scheduledStartTime
      });
    } catch (error) {
      console.error("Failed to create scheduled agent run:", error);
      throw error;
    }
  },

  /**
   * Get a list of all scheduled agent runs
   * @returns Promise resolving to array of scheduled runs
   */
  async getScheduledAgentRuns(): Promise<AgentRunWithMetrics[]> {
    try {
      return await invoke<AgentRunWithMetrics[]>('get_scheduled_agent_runs');
    } catch (error) {
      console.error("Failed to get scheduled agent runs:", error);
      throw error;
    }
  },

  /**
   * Cancel a scheduled agent run
   * @param runId - The run ID to cancel
   * @returns Promise resolving when schedule is cancelled
   */
  async cancelScheduledAgentRun(runId: number): Promise<void> {
    try {
      await invoke('cancel_scheduled_agent_run', { runId });
    } catch (error) {
      console.error("Failed to cancel scheduled agent run:", error);
      throw error;
    }
  },

  /**
   * Lists agent runs with metrics
   * @param agentId - Optional agent ID to filter runs
   * @returns Promise resolving to an array of agent runs with metrics
   */
  async listAgentRuns(agentId?: number): Promise<AgentRunWithMetrics[]> {
    try {
      return await invoke<AgentRunWithMetrics[]>('list_agent_runs', { agentId });
    } catch (error) {
      console.error("Failed to list agent runs:", error);
      // Return empty array instead of throwing to prevent UI crashes
      return [];
    }
  },

  /**
   * Gets a single agent run by ID with metrics
   * @param id - The run ID
   * @returns Promise resolving to the agent run with metrics
   */
  async getAgentRun(id: number): Promise<AgentRunWithMetrics> {
    try {
      return await invoke<AgentRunWithMetrics>('get_agent_run', { id });
    } catch (error) {
      console.error("Failed to get agent run:", error);
      throw new Error(`Failed to get agent run: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  },

  /**
   * Gets a single agent run by ID with real-time metrics from JSONL
   * @param id - The run ID
   * @returns Promise resolving to the agent run with metrics
   */
  async getAgentRunWithRealTimeMetrics(id: number): Promise<AgentRunWithMetrics> {
    try {
      return await invoke<AgentRunWithMetrics>('get_agent_run_with_real_time_metrics', { id });
    } catch (error) {
      console.error("Failed to get agent run with real-time metrics:", error);
      throw new Error(`Failed to get agent run with real-time metrics: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  },

  /**
   * Lists all currently running agent sessions
   * @returns Promise resolving to list of running agent sessions
   */
  async listRunningAgentSessions(): Promise<AgentRun[]> {
    try {
      return await invoke<AgentRun[]>('list_running_sessions');
    } catch (error) {
      console.error("Failed to list running agent sessions:", error);
      throw new Error(`Failed to list running agent sessions: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  },

  /**
   * Resume a paused agent session
   * @param runId - The run ID to resume
   * @returns Promise resolving to the new run ID
   */
  async resumeAgent(runId: number): Promise<number> {
    try {
      return await invoke<number>('resume_agent', { runId });
    } catch (error) {
      console.error("Failed to resume agent:", error);
      throw new Error(`Failed to resume agent: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  },

  /**
   * Lists all currently running agent sessions with metrics
   * @returns Promise resolving to list of running agent sessions with metrics
   */
  async listRunningAgentSessionsWithMetrics(): Promise<AgentRunWithMetrics[]> {
    try {
      return await invoke<AgentRunWithMetrics[]>('list_running_sessions_with_metrics');
    } catch (error) {
      console.error("Failed to list running agent sessions with metrics:", error);
      throw new Error(`Failed to list running agent sessions with metrics: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  },

  /**
   * Kills a running agent session
   * @param runId - The run ID to kill
   * @returns Promise resolving to whether the session was successfully killed
   */
  async killAgentSession(runId: number): Promise<boolean> {
    try {
      return await invoke<boolean>('kill_agent_session', { runId });
    } catch (error) {
      console.error("Failed to kill agent session:", error);
      throw new Error(`Failed to kill agent session: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  },

  /**
   * Gets the status of a specific agent session
   * @param runId - The run ID to check
   * @returns Promise resolving to the session status or null if not found
   */
  async getSessionStatus(runId: number): Promise<string | null> {
    try {
      return await invoke<string | null>('get_session_status', { runId });
    } catch (error) {
      console.error("Failed to get session status:", error);
      throw new Error(`Failed to get session status: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  },

  /**
   * Cleanup finished processes and update their status
   * @returns Promise resolving to list of run IDs that were cleaned up
   */
  async cleanupFinishedProcesses(): Promise<number[]> {
    try {
      return await invoke<number[]>('cleanup_finished_processes');
    } catch (error) {
      console.error("Failed to cleanup finished processes:", error);
      throw new Error(`Failed to cleanup finished processes: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  },

  /**
   * Get real-time output for a running session (with live output fallback)
   * @param runId - The run ID to get output for
   * @returns Promise resolving to the current session output (JSONL format)
   */
  async getSessionOutput(runId: number): Promise<string> {
    try {
      return await invoke<string>('get_session_output', { runId });
    } catch (error) {
      console.error("Failed to get session output:", error);
      throw new Error(`Failed to get session output: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  },

  /**
   * Get live output directly from process stdout buffer
   * @param runId - The run ID to get live output for
   * @returns Promise resolving to the current live output
   */
  async getLiveSessionOutput(runId: number): Promise<string> {
    try {
      return await invoke<string>('get_live_session_output', { runId });
    } catch (error) {
      console.error("Failed to get live session output:", error);
      throw new Error(`Failed to get live session output: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  },

  /**
   * Start streaming real-time output for a running session
   * @param runId - The run ID to stream output for
   * @returns Promise that resolves when streaming starts
   */
  async streamSessionOutput(runId: number): Promise<void> {
    try {
      return await invoke<void>('stream_session_output', { runId });
    } catch (error) {
      console.error("Failed to start streaming session output:", error);
      throw new Error(`Failed to start streaming session output: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  },
};