import { z } from 'zod';
import { BaseService } from './base.service';
import { TAURI_COMMANDS } from '@/constants';
import type { Session } from '@/lib/api.types';

// Zod schemas for type validation
const SessionSchema = z.object({
  id: z.string(),
  project_id: z.string(),
  name: z.string(),
  created_at: z.string(),
  updated_at: z.string(),
  pid: z.number().nullable(),
  exit_code: z.number().nullable(),
  finished_at: z.string().nullable(),
  cancelled_at: z.string().nullable(),
  output_preview: z.string().nullable(),
  message_count: z.number(),
  total_tokens: z.number(),
  last_checkpoint_id: z.string().nullable(),
  auto_checkpoint_count: z.number(),
  checkpoint_id: z.string().nullable(),
  output_path: z.string().nullable(),
});

const SessionArraySchema = z.array(SessionSchema);
const StringSchema = z.string();
const MessageArraySchema = z.array(z.any()); // TODO: Define proper message schema
const VoidSchema = z.void();

/**
 * Service for Claude session operations
 */
export class SessionService extends BaseService {
  constructor(config?: Partial<ServiceConfig>) {
    super({ 
      serviceName: 'SessionService',
      enableLogging: true,
      ...config
    });
  }
  /**
   * Retrieves sessions for a specific project
   * @param projectId - The ID of the project to retrieve sessions for
   * @returns Promise resolving to an array of sessions
   */
  async getProjectSessions(projectId: string): Promise<Session[]> {
    return this.invoke(
      TAURI_COMMANDS.GET_PROJECT_SESSIONS,
      { projectId },
      SessionArraySchema
    );
  }

  /**
   * Opens a new Claude Code session
   * @param path - Optional path to open the session in
   * @returns Promise resolving when the session is opened
   */
  async openNewSession(path?: string): Promise<string> {
    return this.invoke(
      TAURI_COMMANDS.OPEN_NEW_SESSION,
      { path },
      StringSchema
    );
  }

  /**
   * Loads the JSONL history for a specific session
   */
  async loadSessionHistory(sessionId: string, projectId: string): Promise<unknown[]> {
    return this.invoke(
      TAURI_COMMANDS.LOAD_SESSION_HISTORY,
      { sessionId, projectId },
      MessageArraySchema
    );
  }

  /**
   * Tracks a batch of messages for a session for checkpointing
   */
  async trackSessionMessages(
    sessionId: string, 
    projectId: string, 
    projectPath: string, 
    messages: string[]
  ): Promise<void> {
    return this.invokeVoid(
      TAURI_COMMANDS.TRACK_SESSION_MESSAGES,
      { sessionId, projectId, projectPath, messages }
    );
  }
}

// Export singleton instance
export const sessionService = new SessionService();