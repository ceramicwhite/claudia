/**
 * Agent-related Zod schemas
 */

import { z } from 'zod';
import { DateStringSchema, NonEmptyStringSchema, nullable, TokenUsageSchema } from './common';

/**
 * Agent status enum
 */
export const AgentStatusSchema = z.enum(['idle', 'running', 'stopped', 'error']);

/**
 * Agent run status enum
 */
export const AgentRunStatusSchema = z.enum(['pending', 'running', 'completed', 'failed', 'cancelled']);

/**
 * Schedule frequency enum
 */
export const ScheduleFrequencySchema = z.enum(['once', 'hourly', 'daily', 'weekly', 'monthly']);

/**
 * Agent schema
 */
export const AgentSchema = z.object({
  id: z.string(),
  name: NonEmptyStringSchema,
  description: nullable(z.string()),
  instructions: z.string(),
  capabilities: z.array(z.string()).default([]),
  created_at: DateStringSchema,
  updated_at: DateStringSchema,
  last_run: nullable(DateStringSchema),
  run_count: z.number().int().nonnegative().default(0),
  is_system: z.boolean().default(false),
  sandbox_profile_id: nullable(z.string()),
  metadata: z.record(z.unknown()).default({}),
});

export type Agent = z.infer<typeof AgentSchema>;

/**
 * Agent run schema
 */
export const AgentRunSchema = z.object({
  id: z.string(),
  agent_id: z.string(),
  session_id: z.string(),
  status: AgentRunStatusSchema,
  started_at: DateStringSchema,
  completed_at: nullable(DateStringSchema),
  error: nullable(z.string()),
  input: z.string(),
  output: nullable(z.string()),
  token_usage: nullable(TokenUsageSchema),
  duration_ms: nullable(z.number().int().nonnegative()),
  metadata: z.record(z.unknown()).default({}),
});

export type AgentRun = z.infer<typeof AgentRunSchema>;

/**
 * Running session schema
 */
export const RunningSessionSchema = z.object({
  id: z.string(),
  agent_id: z.string(),
  agent_name: z.string(),
  project_path: z.string(),
  started_at: DateStringSchema,
  pid: z.number().int().positive(),
  status: z.string(),
});

export type RunningSession = z.infer<typeof RunningSessionSchema>;

/**
 * Session metrics schema
 */
export const SessionMetricsSchema = z.object({
  cpu_percent: z.number().nonnegative(),
  memory_mb: z.number().nonnegative(),
  duration_seconds: z.number().nonnegative(),
  message_count: z.number().int().nonnegative(),
  last_activity: DateStringSchema,
});

export type SessionMetrics = z.infer<typeof SessionMetricsSchema>;

/**
 * Running session with metrics
 */
export const RunningSessionWithMetricsSchema = RunningSessionSchema.extend({
  metrics: SessionMetricsSchema,
});

export type RunningSessionWithMetrics = z.infer<typeof RunningSessionWithMetricsSchema>;

/**
 * Create agent params
 */
export const CreateAgentParamsSchema = z.object({
  name: NonEmptyStringSchema,
  description: z.string().optional(),
  instructions: NonEmptyStringSchema,
  capabilities: z.array(z.string()).optional(),
  sandbox_profile_id: z.string().optional(),
  metadata: z.record(z.unknown()).optional(),
});

export type CreateAgentParams = z.infer<typeof CreateAgentParamsSchema>;

/**
 * Update agent params
 */
export const UpdateAgentParamsSchema = z.object({
  name: NonEmptyStringSchema.optional(),
  description: z.string().optional(),
  instructions: z.string().optional(),
  capabilities: z.array(z.string()).optional(),
  sandbox_profile_id: z.string().optional(),
  metadata: z.record(z.unknown()).optional(),
});

export type UpdateAgentParams = z.infer<typeof UpdateAgentParamsSchema>;

/**
 * Execute agent params
 */
export const ExecuteAgentParamsSchema = z.object({
  agent_id: z.string(),
  input: NonEmptyStringSchema,
  project_path: z.string(),
  env_vars: z.record(z.string()).optional(),
  metadata: z.record(z.unknown()).optional(),
});

export type ExecuteAgentParams = z.infer<typeof ExecuteAgentParamsSchema>;

/**
 * Scheduled agent run schema
 */
export const ScheduledAgentRunSchema = z.object({
  id: z.string(),
  agent_id: z.string(),
  project_path: z.string(),
  input: z.string(),
  frequency: ScheduleFrequencySchema,
  next_run: DateStringSchema,
  last_run: nullable(DateStringSchema),
  created_at: DateStringSchema,
  enabled: z.boolean(),
  metadata: z.record(z.unknown()).default({}),
});

export type ScheduledAgentRun = z.infer<typeof ScheduledAgentRunSchema>;

/**
 * Create scheduled run params
 */
export const CreateScheduledRunParamsSchema = z.object({
  agent_id: z.string(),
  project_path: z.string(),
  input: NonEmptyStringSchema,
  frequency: ScheduleFrequencySchema,
  start_time: DateStringSchema.optional(),
  enabled: z.boolean().default(true),
  metadata: z.record(z.unknown()).optional(),
});

export type CreateScheduledRunParams = z.infer<typeof CreateScheduledRunParamsSchema>;

/**
 * GitHub agent metadata schema
 */
export const GitHubAgentMetadataSchema = z.object({
  name: z.string(),
  path: z.string(),
  url: z.string(),
  sha: z.string(),
  size: z.number(),
  description: z.string().optional(),
});

export type GitHubAgentMetadata = z.infer<typeof GitHubAgentMetadataSchema>;

/**
 * Transform functions
 */
export const transformAgent = (data: unknown): Agent => {
  if (typeof data === 'object' && data !== null) {
    const obj = data as Record<string, unknown>;
    
    // Ensure dates are in ISO format
    if (obj.created_at && typeof obj.created_at === 'string') {
      obj.created_at = new Date(obj.created_at).toISOString();
    }
    if (obj.updated_at && typeof obj.updated_at === 'string') {
      obj.updated_at = new Date(obj.updated_at).toISOString();
    }
    if (obj.last_run && typeof obj.last_run === 'string') {
      obj.last_run = new Date(obj.last_run).toISOString();
    }
    
    // Ensure arrays
    if (!Array.isArray(obj.capabilities)) {
      obj.capabilities = [];
    }
    
    // Ensure metadata object
    if (!obj.metadata || typeof obj.metadata !== 'object') {
      obj.metadata = {};
    }
    
    // Ensure number fields
    if (typeof obj.run_count !== 'number') {
      obj.run_count = 0;
    }
  }
  
  return AgentSchema.parse(data);
};

export const transformAgentRun = (data: unknown): AgentRun => {
  if (typeof data === 'object' && data !== null) {
    const obj = data as Record<string, unknown>;
    
    // Ensure dates are in ISO format
    if (obj.started_at && typeof obj.started_at === 'string') {
      obj.started_at = new Date(obj.started_at).toISOString();
    }
    if (obj.completed_at && typeof obj.completed_at === 'string') {
      obj.completed_at = new Date(obj.completed_at).toISOString();
    }
    
    // Ensure metadata object
    if (!obj.metadata || typeof obj.metadata !== 'object') {
      obj.metadata = {};
    }
  }
  
  return AgentRunSchema.parse(data);
};