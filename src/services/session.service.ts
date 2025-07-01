import { z } from 'zod';
import { BaseService, ServiceConfig } from './base.service';
import { TAURI_COMMANDS } from '@/constants';
import type { Session } from '@/lib/api.types';

// Zod schemas for type validation
const SessionSchema = z.object({
  id: z.string(),
  project_id: z.string(),
  project_path: z.string(),
  todo_data: z.any().optional(),
  created_at: z.number(),
  first_message: z.string().optional(),
  message_timestamp: z.string().optional(),
});

const SessionArraySchema = z.array(SessionSchema);
const StringSchema = z.string();
const MessageArraySchema = z.array(z.any()); // TODO: Define proper message schema

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
    return this.invoke<{ projectId: string }, Session[]>(
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
    return this.invoke<{ path?: string }, string>(
      TAURI_COMMANDS.OPEN_NEW_SESSION,
      { path },
      StringSchema
    );
  }

  /**
   * Loads the JSONL history for a specific session
   */
  async loadSessionHistory(sessionId: string, projectId: string): Promise<unknown[]> {
    return this.invoke<{ sessionId: string; projectId: string }, unknown[]>(
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
    return this.invokeVoid<{ sessionId: string; projectId: string; projectPath: string; messages: string[] }>(
      TAURI_COMMANDS.TRACK_SESSION_MESSAGES,
      { sessionId, projectId, projectPath, messages }
    );
  }
}

// Export singleton instance
export const sessionService = new SessionService();