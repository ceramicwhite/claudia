import { invoke } from "@tauri-apps/api/core";
import type {
  SandboxProfile,
  SandboxRule,
  PlatformCapabilities,
  SandboxViolation,
  SandboxViolationStats,
  SandboxProfileExport,
  SandboxImportResult
} from "../types";

/**
 * Sandbox API module
 */
export const sandboxApi = {
  /**
   * Lists all sandbox profiles
   * @returns Promise resolving to an array of sandbox profiles
   */
  async listSandboxProfiles(): Promise<SandboxProfile[]> {
    try {
      return await invoke<SandboxProfile[]>('list_sandbox_profiles');
    } catch (error) {
      console.error("Failed to list sandbox profiles:", error);
      throw error;
    }
  },

  /**
   * Creates a new sandbox profile
   * @param name - The profile name
   * @param description - Optional description
   * @returns Promise resolving to the created profile
   */
  async createSandboxProfile(name: string, description?: string): Promise<SandboxProfile> {
    try {
      return await invoke<SandboxProfile>('create_sandbox_profile', { name, description });
    } catch (error) {
      console.error("Failed to create sandbox profile:", error);
      throw error;
    }
  },

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
    try {
      return await invoke<SandboxProfile>('update_sandbox_profile', { 
        id, 
        name, 
        description, 
        is_active, 
        is_default 
      });
    } catch (error) {
      console.error("Failed to update sandbox profile:", error);
      throw error;
    }
  },

  /**
   * Deletes a sandbox profile
   * @param id - The profile ID to delete
   * @returns Promise resolving when the profile is deleted
   */
  async deleteSandboxProfile(id: number): Promise<void> {
    try {
      return await invoke('delete_sandbox_profile', { id });
    } catch (error) {
      console.error("Failed to delete sandbox profile:", error);
      throw error;
    }
  },

  /**
   * Gets a single sandbox profile by ID
   * @param id - The profile ID
   * @returns Promise resolving to the profile
   */
  async getSandboxProfile(id: number): Promise<SandboxProfile> {
    try {
      return await invoke<SandboxProfile>('get_sandbox_profile', { id });
    } catch (error) {
      console.error("Failed to get sandbox profile:", error);
      throw error;
    }
  },

  /**
   * Lists rules for a sandbox profile
   * @param profileId - The profile ID
   * @returns Promise resolving to an array of rules
   */
  async listSandboxRules(profileId: number): Promise<SandboxRule[]> {
    try {
      return await invoke<SandboxRule[]>('list_sandbox_rules', { profile_id: profileId });
    } catch (error) {
      console.error("Failed to list sandbox rules:", error);
      throw error;
    }
  },

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
    try {
      return await invoke<SandboxRule>('create_sandbox_rule', { 
        profile_id: profileId,
        operation_type,
        pattern_type,
        pattern_value,
        enabled,
        platform_support
      });
    } catch (error) {
      console.error("Failed to create sandbox rule:", error);
      throw error;
    }
  },

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
    try {
      return await invoke<SandboxRule>('update_sandbox_rule', { 
        id,
        operation_type,
        pattern_type,
        pattern_value,
        enabled,
        platform_support
      });
    } catch (error) {
      console.error("Failed to update sandbox rule:", error);
      throw error;
    }
  },

  /**
   * Deletes a sandbox rule
   * @param id - The rule ID to delete
   * @returns Promise resolving when the rule is deleted
   */
  async deleteSandboxRule(id: number): Promise<void> {
    try {
      return await invoke('delete_sandbox_rule', { id });
    } catch (error) {
      console.error("Failed to delete sandbox rule:", error);
      throw error;
    }
  },

  /**
   * Gets platform capabilities for sandbox configuration
   * @returns Promise resolving to platform capabilities
   */
  async getPlatformCapabilities(): Promise<PlatformCapabilities> {
    try {
      return await invoke<PlatformCapabilities>('get_platform_capabilities');
    } catch (error) {
      console.error("Failed to get platform capabilities:", error);
      throw error;
    }
  },

  /**
   * Tests a sandbox profile
   * @param profileId - The profile ID to test
   * @returns Promise resolving to test result message
   */
  async testSandboxProfile(profileId: number): Promise<string> {
    try {
      return await invoke<string>('test_sandbox_profile', { profile_id: profileId });
    } catch (error) {
      console.error("Failed to test sandbox profile:", error);
      throw error;
    }
  },

  // Sandbox violation methods

  /**
   * Lists sandbox violations with optional filtering
   * @param profileId - Optional profile ID to filter by
   * @param agentId - Optional agent ID to filter by
   * @param limit - Optional limit on number of results
   * @returns Promise resolving to array of violations
   */
  async listSandboxViolations(profileId?: number, agentId?: number, limit?: number): Promise<SandboxViolation[]> {
    try {
      return await invoke<SandboxViolation[]>('list_sandbox_violations', { 
        profile_id: profileId, 
        agent_id: agentId, 
        limit 
      });
    } catch (error) {
      console.error("Failed to list sandbox violations:", error);
      throw error;
    }
  },

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
    try {
      return await invoke('log_sandbox_violation', {
        profile_id: violation.profileId,
        agent_id: violation.agentId,
        agent_run_id: violation.agentRunId,
        operation_type: violation.operationType,
        pattern_value: violation.patternValue,
        process_name: violation.processName,
        pid: violation.pid
      });
    } catch (error) {
      console.error("Failed to log sandbox violation:", error);
      throw error;
    }
  },

  /**
   * Clears old sandbox violations
   * @param olderThanDays - Optional days to keep (clears all if not specified)
   * @returns Promise resolving to number of deleted violations
   */
  async clearSandboxViolations(olderThanDays?: number): Promise<number> {
    try {
      return await invoke<number>('clear_sandbox_violations', { older_than_days: olderThanDays });
    } catch (error) {
      console.error("Failed to clear sandbox violations:", error);
      throw error;
    }
  },

  /**
   * Gets sandbox violation statistics
   * @returns Promise resolving to violation stats
   */
  async getSandboxViolationStats(): Promise<SandboxViolationStats> {
    try {
      return await invoke<SandboxViolationStats>('get_sandbox_violation_stats');
    } catch (error) {
      console.error("Failed to get sandbox violation stats:", error);
      throw error;
    }
  },

  // Import/Export methods

  /**
   * Exports a single sandbox profile with its rules
   * @param profileId - The profile ID to export
   * @returns Promise resolving to export data
   */
  async exportSandboxProfile(profileId: number): Promise<SandboxProfileExport> {
    try {
      return await invoke<SandboxProfileExport>('export_sandbox_profile', { profile_id: profileId });
    } catch (error) {
      console.error("Failed to export sandbox profile:", error);
      throw error;
    }
  },

  /**
   * Exports all sandbox profiles
   * @returns Promise resolving to export data
   */
  async exportAllSandboxProfiles(): Promise<SandboxProfileExport> {
    try {
      return await invoke<SandboxProfileExport>('export_all_sandbox_profiles');
    } catch (error) {
      console.error("Failed to export all sandbox profiles:", error);
      throw error;
    }
  },

  /**
   * Imports sandbox profiles from export data
   * @param exportData - The export data to import
   * @returns Promise resolving to import results
   */
  async importSandboxProfiles(exportData: SandboxProfileExport): Promise<SandboxImportResult[]> {
    try {
      return await invoke<SandboxImportResult[]>('import_sandbox_profiles', { export_data: exportData });
    } catch (error) {
      console.error("Failed to import sandbox profiles:", error);
      throw error;
    }
  },
};