import { BaseService, ServiceConfig } from './base.service';
import { TAURI_COMMANDS, ERROR_MESSAGES } from '@/constants';
import type { 
  SandboxProfile,
  SandboxRule,
  PlatformCapabilities,
  SandboxViolation,
  SandboxViolationStats,
  SandboxProfileExport,
  ImportResult
} from '@/lib/api.types';

/**
 * Service for sandbox operations
 */
export class SandboxService extends BaseService {
  constructor(config?: Partial<ServiceConfig>) {
    super({
      serviceName: 'SandboxService',
      enableLogging: true,
      ...config
    });
  }

  /**
   * Lists all sandbox profiles
   * @returns Promise resolving to an array of sandbox profiles
   */
  async listSandboxProfiles(): Promise<SandboxProfile[]> {
    return this.invokeNoArgs<SandboxProfile[]>(TAURI_COMMANDS.LIST_SANDBOX_PROFILES, ERROR_MESSAGES.FAILED_TO_LIST_SANDBOX_PROFILES);
  }

  /**
   * Creates a new sandbox profile
   * @param name - The profile name
   * @param description - Optional description
   * @returns Promise resolving to the created profile
   */
  async createSandboxProfile(name: string, description?: string): Promise<SandboxProfile> {
    return this.invoke<SandboxProfile>(
      TAURI_COMMANDS.CREATE_SANDBOX_PROFILE,
      { name, description },
      ERROR_MESSAGES.FAILED_TO_CREATE_SANDBOX_PROFILE
    );
  }

  /**
   * Updates a sandbox profile
   * @param id - The profile ID
   * @param name - The updated name
   * @param description - Optional description
   * @param is_active - Whether the profile is active
   * @param is_default - Whether the profile is the default
   * @returns Promise resolving to the updated profile
   */
  async updateSandboxProfile(
    id: number, 
    name: string, 
    description: string | undefined, 
    is_active: boolean, 
    is_default: boolean
  ): Promise<SandboxProfile> {
    return this.invoke<SandboxProfile>(
      TAURI_COMMANDS.UPDATE_SANDBOX_PROFILE,
      { id, name, description, is_active, is_default },
      ERROR_MESSAGES.FAILED_TO_UPDATE_SANDBOX_PROFILE
    );
  }

  /**
   * Deletes a sandbox profile
   * @param id - The profile ID to delete
   * @returns Promise resolving when the profile is deleted
   */
  async deleteSandboxProfile(id: number): Promise<void> {
    return this.invoke<void>(
      TAURI_COMMANDS.DELETE_SANDBOX_PROFILE,
      { id },
      ERROR_MESSAGES.FAILED_TO_DELETE_SANDBOX_PROFILE
    );
  }

  /**
   * Gets a single sandbox profile by ID
   * @param id - The profile ID
   * @returns Promise resolving to the profile
   */
  async getSandboxProfile(id: number): Promise<SandboxProfile> {
    return this.invoke<SandboxProfile>(
      TAURI_COMMANDS.GET_SANDBOX_PROFILE,
      { id },
      ERROR_MESSAGES.FAILED_TO_GET_SANDBOX_PROFILE
    );
  }

  /**
   * Lists rules for a sandbox profile
   * @param profileId - The profile ID
   * @returns Promise resolving to an array of rules
   */
  async listSandboxRules(profileId: number): Promise<SandboxRule[]> {
    return this.invoke<SandboxRule[]>(
      TAURI_COMMANDS.LIST_SANDBOX_RULES,
      { profile_id: profileId },
      ERROR_MESSAGES.FAILED_TO_LIST_SANDBOX_RULES
    );
  }

  /**
   * Creates a new sandbox rule
   * @param profileId - The profile ID
   * @param operation_type - The operation type
   * @param pattern_type - The pattern type
   * @param pattern_value - The pattern value
   * @param enabled - Whether the rule is enabled
   * @param platform_support - Optional platform support JSON
   * @returns Promise resolving to the created rule
   */
  async createSandboxRule(
    profileId: number,
    operation_type: string,
    pattern_type: string,
    pattern_value: string,
    enabled: boolean,
    platform_support?: string
  ): Promise<SandboxRule> {
    return this.invoke<SandboxRule>(
      TAURI_COMMANDS.CREATE_SANDBOX_RULE,
      { 
        profile_id: profileId,
        operation_type,
        pattern_type,
        pattern_value,
        enabled,
        platform_support
      },
      ERROR_MESSAGES.FAILED_TO_CREATE_SANDBOX_RULE
    );
  }

  /**
   * Updates a sandbox rule
   * @param id - The rule ID
   * @param operation_type - The operation type
   * @param pattern_type - The pattern type
   * @param pattern_value - The pattern value
   * @param enabled - Whether the rule is enabled
   * @param platform_support - Optional platform support JSON
   * @returns Promise resolving to the updated rule
   */
  async updateSandboxRule(
    id: number,
    operation_type: string,
    pattern_type: string,
    pattern_value: string,
    enabled: boolean,
    platform_support?: string
  ): Promise<SandboxRule> {
    return this.invoke<SandboxRule>(
      TAURI_COMMANDS.UPDATE_SANDBOX_RULE,
      { 
        id,
        operation_type,
        pattern_type,
        pattern_value,
        enabled,
        platform_support
      },
      ERROR_MESSAGES.FAILED_TO_UPDATE_SANDBOX_RULE
    );
  }

  /**
   * Deletes a sandbox rule
   * @param id - The rule ID to delete
   * @returns Promise resolving when the rule is deleted
   */
  async deleteSandboxRule(id: number): Promise<void> {
    return this.invoke<void>(
      TAURI_COMMANDS.DELETE_SANDBOX_RULE,
      { id },
      ERROR_MESSAGES.FAILED_TO_DELETE_SANDBOX_RULE
    );
  }

  /**
   * Gets platform capabilities for sandbox configuration
   * @returns Promise resolving to platform capabilities
   */
  async getPlatformCapabilities(): Promise<PlatformCapabilities> {
    return this.invokeNoArgs<PlatformCapabilities>(TAURI_COMMANDS.GET_PLATFORM_CAPABILITIES, ERROR_MESSAGES.FAILED_TO_GET_PLATFORM_CAPABILITIES);
  }

  /**
   * Tests a sandbox profile
   * @param profileId - The profile ID to test
   * @returns Promise resolving to test result message
   */
  async testSandboxProfile(profileId: number): Promise<string> {
    return this.invoke<string>(
      TAURI_COMMANDS.TEST_SANDBOX_PROFILE,
      { profile_id: profileId },
      ERROR_MESSAGES.FAILED_TO_TEST_SANDBOX_PROFILE
    );
  }

  /**
   * Lists sandbox violations with optional filtering
   * @param profileId - Optional profile ID to filter by
   * @param agentId - Optional agent ID to filter by
   * @param limit - Optional limit on number of results
   * @returns Promise resolving to array of violations
   */
  async listSandboxViolations(profileId?: number, agentId?: number, limit?: number): Promise<SandboxViolation[]> {
    return this.invoke<SandboxViolation[]>(
      TAURI_COMMANDS.LIST_SANDBOX_VIOLATIONS,
      { profile_id: profileId, agent_id: agentId, limit },
      ERROR_MESSAGES.FAILED_TO_LIST_SANDBOX_VIOLATIONS
    );
  }

  /**
   * Logs a sandbox violation
   * @param violation - The violation details
   * @returns Promise resolving when logged
   */
  async logSandboxViolation(violation: {
    profileId?: number;
    agentId?: number;
    agentRunId?: number;
    operationType: string;
    patternValue?: string;
    processName?: string;
    pid?: number;
  }): Promise<void> {
    return this.invoke<void>(
      TAURI_COMMANDS.LOG_SANDBOX_VIOLATION,
      {
        profile_id: violation.profileId,
        agent_id: violation.agentId,
        agent_run_id: violation.agentRunId,
        operation_type: violation.operationType,
        pattern_value: violation.patternValue,
        process_name: violation.processName,
        pid: violation.pid
      },
      ERROR_MESSAGES.FAILED_TO_LOG_SANDBOX_VIOLATION
    );
  }

  /**
   * Clears old sandbox violations
   * @param olderThanDays - Optional days to keep (clears all if not specified)
   * @returns Promise resolving to number of deleted violations
   */
  async clearSandboxViolations(olderThanDays?: number): Promise<number> {
    return this.invoke<number>(
      TAURI_COMMANDS.CLEAR_SANDBOX_VIOLATIONS,
      { older_than_days: olderThanDays },
      ERROR_MESSAGES.FAILED_TO_CLEAR_SANDBOX_VIOLATIONS
    );
  }

  /**
   * Gets sandbox violation statistics
   * @returns Promise resolving to violation stats
   */
  async getSandboxViolationStats(): Promise<SandboxViolationStats> {
    return this.invokeNoArgs<SandboxViolationStats>(TAURI_COMMANDS.GET_SANDBOX_VIOLATION_STATS, ERROR_MESSAGES.FAILED_TO_GET_SANDBOX_VIOLATION_STATS);
  }

  /**
   * Exports a single sandbox profile with its rules
   * @param profileId - The profile ID to export
   * @returns Promise resolving to export data
   */
  async exportSandboxProfile(profileId: number): Promise<SandboxProfileExport> {
    return this.invoke<SandboxProfileExport>(
      TAURI_COMMANDS.EXPORT_SANDBOX_PROFILE,
      { profile_id: profileId },
      ERROR_MESSAGES.FAILED_TO_EXPORT_SANDBOX_PROFILE
    );
  }

  /**
   * Exports all sandbox profiles
   * @returns Promise resolving to export data
   */
  async exportAllSandboxProfiles(): Promise<SandboxProfileExport> {
    return this.invokeNoArgs<SandboxProfileExport>(TAURI_COMMANDS.EXPORT_ALL_SANDBOX_PROFILES, ERROR_MESSAGES.FAILED_TO_EXPORT_ALL_SANDBOX_PROFILES);
  }

  /**
   * Imports sandbox profiles from export data
   * @param exportData - The export data to import
   * @returns Promise resolving to import results
   */
  async importSandboxProfiles(exportData: SandboxProfileExport): Promise<ImportResult[]> {
    return this.invoke<ImportResult[]>(
      TAURI_COMMANDS.IMPORT_SANDBOX_PROFILES,
      { export_data: exportData },
      ERROR_MESSAGES.FAILED_TO_IMPORT_SANDBOX_PROFILES
    );
  }
}

// Export singleton instance
export const sandboxService = new SandboxService();