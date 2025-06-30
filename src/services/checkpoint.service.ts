import { BaseService, ServiceConfig } from './base.service';
import { TAURI_COMMANDS, ERROR_MESSAGES } from '@/constants';
import type { 
  CheckpointResult,
  Checkpoint,
  SessionTimeline,
  CheckpointStrategy,
  CheckpointDiff
} from '@/lib/api.types';

/**
 * Service for checkpoint operations
 */
export class CheckpointService extends BaseService {
  constructor(config?: Partial<ServiceConfig>) {
    super({
      serviceName: 'CheckpointService',
      enableLogging: true,
      ...config
    });
  }

  /**
   * Creates a checkpoint for the current session state
   */
  async createCheckpoint(
    sessionId: string,
    projectId: string,
    projectPath: string,
    messageIndex?: number,
    description?: string
  ): Promise<CheckpointResult> {
    return this.invoke<CheckpointResult>(
      TAURI_COMMANDS.CREATE_CHECKPOINT,
      { sessionId, projectId, projectPath, messageIndex, description }
    );
  }

  /**
   * Restores a session to a specific checkpoint
   */
  async restoreCheckpoint(
    checkpointId: string,
    sessionId: string,
    projectId: string,
    projectPath: string
  ): Promise<CheckpointResult> {
    return this.invoke<CheckpointResult>(
      TAURI_COMMANDS.RESTORE_CHECKPOINT,
      { checkpointId, sessionId, projectId, projectPath }
    );
  }

  /**
   * Lists all checkpoints for a session
   */
  async listCheckpoints(
    sessionId: string,
    projectId: string,
    projectPath: string
  ): Promise<Checkpoint[]> {
    return this.invoke<Checkpoint[]>(
      TAURI_COMMANDS.LIST_CHECKPOINTS,
      { sessionId, projectId, projectPath }
    );
  }

  /**
   * Forks a new timeline branch from a checkpoint
   */
  async forkFromCheckpoint(
    checkpointId: string,
    sessionId: string,
    projectId: string,
    projectPath: string,
    newSessionId: string,
    description?: string
  ): Promise<CheckpointResult> {
    return this.invoke<CheckpointResult>(
      TAURI_COMMANDS.FORK_FROM_CHECKPOINT,
      { checkpointId, sessionId, projectId, projectPath, newSessionId, description }
    );
  }

  /**
   * Gets the timeline for a session
   */
  async getSessionTimeline(
    sessionId: string,
    projectId: string,
    projectPath: string
  ): Promise<SessionTimeline> {
    return this.invoke<SessionTimeline>(
      TAURI_COMMANDS.GET_SESSION_TIMELINE,
      { sessionId, projectId, projectPath }
    );
  }

  /**
   * Updates checkpoint settings for a session
   */
  async updateCheckpointSettings(
    sessionId: string,
    projectId: string,
    projectPath: string,
    autoCheckpointEnabled: boolean,
    checkpointStrategy: CheckpointStrategy
  ): Promise<void> {
    return this.invoke<void>(
      TAURI_COMMANDS.UPDATE_CHECKPOINT_SETTINGS,
      { sessionId, projectId, projectPath, autoCheckpointEnabled, checkpointStrategy }
    );
  }

  /**
   * Gets diff between two checkpoints
   */
  async getCheckpointDiff(
    fromCheckpointId: string,
    toCheckpointId: string,
    sessionId: string,
    projectId: string
  ): Promise<CheckpointDiff> {
    return this.invoke<CheckpointDiff>(
      TAURI_COMMANDS.GET_CHECKPOINT_DIFF,
      { fromCheckpointId, toCheckpointId, sessionId, projectId },
      ERROR_MESSAGES.FAILED_TO_GET_CHECKPOINT_DIFF
    );
  }

  /**
   * Tracks a message for checkpointing
   */
  async trackCheckpointMessage(
    sessionId: string,
    projectId: string,
    projectPath: string,
    message: string
  ): Promise<void> {
    return this.invoke<void>(
      TAURI_COMMANDS.TRACK_CHECKPOINT_MESSAGE,
      { sessionId, projectId, projectPath, message },
      ERROR_MESSAGES.FAILED_TO_TRACK_CHECKPOINT_MESSAGE
    );
  }

  /**
   * Checks if auto-checkpoint should be triggered
   */
  async checkAutoCheckpoint(
    sessionId: string,
    projectId: string,
    projectPath: string,
    message: string
  ): Promise<boolean> {
    return this.invoke<boolean>(
      TAURI_COMMANDS.CHECK_AUTO_CHECKPOINT,
      { sessionId, projectId, projectPath, message },
      ERROR_MESSAGES.FAILED_TO_CHECK_AUTO_CHECKPOINT
    );
  }

  /**
   * Triggers cleanup of old checkpoints
   */
  async cleanupOldCheckpoints(
    sessionId: string,
    projectId: string,
    projectPath: string,
    keepCount: number
  ): Promise<number> {
    return this.invoke<number>(
      TAURI_COMMANDS.CLEANUP_OLD_CHECKPOINTS,
      { sessionId, projectId, projectPath, keepCount },
      ERROR_MESSAGES.FAILED_TO_CLEANUP_OLD_CHECKPOINTS
    );
  }

  /**
   * Gets checkpoint settings for a session
   */
  async getCheckpointSettings(
    sessionId: string,
    projectId: string,
    projectPath: string
  ): Promise<{
    auto_checkpoint_enabled: boolean;
    checkpoint_strategy: CheckpointStrategy;
    total_checkpoints: number;
    current_checkpoint_id?: string;
  }> {
    return this.invoke(
      TAURI_COMMANDS.GET_CHECKPOINT_SETTINGS,
      { sessionId, projectId, projectPath },
      ERROR_MESSAGES.FAILED_TO_GET_CHECKPOINT_SETTINGS
    );
  }

  /**
   * Clears checkpoint manager for a session (cleanup on session end)
   */
  async clearCheckpointManager(sessionId: string): Promise<void> {
    return this.invoke<void>(
      TAURI_COMMANDS.CLEAR_CHECKPOINT_MANAGER,
      { sessionId },
      ERROR_MESSAGES.FAILED_TO_CLEAR_CHECKPOINT_MANAGER
    );
  }
}

// Export singleton instance
export const checkpointService = new CheckpointService();