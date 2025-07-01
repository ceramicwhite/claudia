import { z } from 'zod';
import { BaseService, ServiceConfig } from './base.service';
import { TAURI_COMMANDS } from '@/constants';
import type { 
  UsageStats,
  UsageEntry,
  ProjectUsage
} from '@/lib/api.types';

// Zod schemas for type validation
const ModelUsageSchema = z.object({
  model: z.string(),
  total_cost: z.number(),
  total_tokens: z.number(),
  input_tokens: z.number(),
  output_tokens: z.number(),
  cache_creation_tokens: z.number(),
  cache_read_tokens: z.number(),
  session_count: z.number(),
});

const DailyUsageSchema = z.object({
  date: z.string(),
  total_cost: z.number(),
  total_tokens: z.number(),
  models_used: z.array(z.string()),
});

const ProjectUsageSchema = z.object({
  project_path: z.string(),
  project_name: z.string(),
  total_cost: z.number(),
  total_tokens: z.number(),
  session_count: z.number(),
  last_used: z.string(),
});

const UsageStatsSchema = z.object({
  total_cost: z.number(),
  total_tokens: z.number(),
  total_input_tokens: z.number(),
  total_output_tokens: z.number(),
  total_cache_creation_tokens: z.number(),
  total_cache_read_tokens: z.number(),
  total_sessions: z.number(),
  by_model: z.array(ModelUsageSchema),
  by_date: z.array(DailyUsageSchema),
  by_project: z.array(ProjectUsageSchema),
});

const UsageEntrySchema = z.object({
  project: z.string(),
  timestamp: z.string(),
  model: z.string(),
  input_tokens: z.number(),
  output_tokens: z.number(),
  cache_write_tokens: z.number(),
  cache_read_tokens: z.number(),
  cost: z.number(),
});

const ProjectUsageArraySchema = z.array(ProjectUsageSchema);
const UsageEntryArraySchema = z.array(UsageEntrySchema);

/**
 * Service for usage tracking operations
 */
export class UsageService extends BaseService {
  constructor(config?: Partial<ServiceConfig>) {
    super({ 
      serviceName: 'UsageService',
      enableLogging: true,
      ...config
    });
  }
  /**
   * Gets overall usage statistics
   * @returns Promise resolving to usage statistics
   */
  async getUsageStats(): Promise<UsageStats> {
    return this.invoke<{}, UsageStats>(
      TAURI_COMMANDS.GET_USAGE_STATS,
      {},
      UsageStatsSchema
    );
  }

  /**
   * Gets usage statistics filtered by date range
   * @param startDate - Start date (ISO format)
   * @param endDate - End date (ISO format)
   * @returns Promise resolving to usage statistics
   */
  async getUsageByDateRange(startDate: string, endDate: string): Promise<UsageStats> {
    return this.invoke<{ startDate: string; endDate: string }, UsageStats>(
      TAURI_COMMANDS.GET_USAGE_BY_DATE_RANGE,
      { startDate, endDate },
      UsageStatsSchema
    );
  }

  /**
   * Gets usage statistics grouped by session
   * @param since - Optional start date (YYYYMMDD)
   * @param until - Optional end date (YYYYMMDD)
   * @param order - Optional sort order ('asc' or 'desc')
   * @returns Promise resolving to an array of session usage data
   */
  async getSessionStats(
    since?: string,
    until?: string,
    order?: "asc" | "desc"
  ): Promise<ProjectUsage[]> {
    return this.invoke<{ since?: string; until?: string; order?: "asc" | "desc" }, ProjectUsage[]>(
      TAURI_COMMANDS.GET_SESSION_STATS,
      { since, until, order },
      ProjectUsageArraySchema
    );
  }

  /**
   * Gets detailed usage entries with optional filtering
   * @param limit - Optional limit for number of entries
   * @returns Promise resolving to array of usage entries
   */
  async getUsageDetails(limit?: number): Promise<UsageEntry[]> {
    return this.invoke<{ limit?: number }, UsageEntry[]>(
      TAURI_COMMANDS.GET_USAGE_DETAILS,
      { limit },
      UsageEntryArraySchema
    );
  }
}

// Export singleton instance
export const usageService = new UsageService();