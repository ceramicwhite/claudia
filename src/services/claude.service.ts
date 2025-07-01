import { BaseService, ServiceConfig } from './base.service';
import { TAURI_COMMANDS } from '@/constants';

/**
 * Service for Claude process operations
 */
export class ClaudeService extends BaseService {
  constructor(config?: Partial<ServiceConfig>) {
    super({
      serviceName: 'ClaudeService',
      enableLogging: true,
      ...config
    });
  }
  /**
   * Executes a new interactive Claude Code session with streaming output
   */
  async executeClaudeCode(projectPath: string, prompt: string, model: string): Promise<void> {
    return this.invoke<{ projectPath: string; prompt: string; model: string }, void>(
      TAURI_COMMANDS.EXECUTE_CLAUDE_CODE,
      { projectPath, prompt, model }
    );
  }

  /**
   * Continues an existing Claude Code conversation with streaming output
   */
  async continueClaudeCode(projectPath: string, prompt: string, model: string): Promise<void> {
    return this.invoke<{ projectPath: string; prompt: string; model: string }, void>(
      TAURI_COMMANDS.CONTINUE_CLAUDE_CODE,
      { projectPath, prompt, model }
    );
  }

  /**
   * Resumes an existing Claude Code session by ID with streaming output
   */
  async resumeClaudeCode(projectPath: string, sessionId: string, prompt: string, model: string): Promise<void> {
    return this.invoke<{ projectPath: string; sessionId: string; prompt: string; model: string }, void>(
      TAURI_COMMANDS.RESUME_CLAUDE_CODE,
      { projectPath, sessionId, prompt, model }
    );
  }

  /**
   * Cancels the currently running Claude Code execution
   * @param sessionId - Optional session ID to cancel a specific session
   */
  async cancelClaudeExecution(sessionId?: string): Promise<void> {
    return this.invoke<{ sessionId?: string }, void>(
      TAURI_COMMANDS.CANCEL_CLAUDE_EXECUTION,
      { sessionId }
    );
  }
}

// Export singleton instance
export const claudeService = new ClaudeService();