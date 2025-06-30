/**
 * Project-related Zod schemas
 */

import { z } from 'zod';
import { DateStringSchema, FilePathSchema, nullable, TokenUsageSchema } from './common';

/**
 * Session status enum
 */
export const SessionStatusSchema = z.enum(['active', 'completed', 'failed']);

/**
 * Session type enum
 */
export const SessionTypeSchema = z.enum(['chat', 'agent']);

/**
 * Project schema
 */
export const ProjectSchema = z.object({
  id: z.string(),
  name: z.string(),
  path: FilePathSchema,
  created_at: DateStringSchema,
  last_accessed: nullable(DateStringSchema),
  description: nullable(z.string()),
  is_git_repo: z.boolean(),
  remote_url: nullable(z.string()),
  branch: nullable(z.string()),
  tags: z.array(z.string()).default([]),
  metadata: z.record(z.unknown()).default({}),
});

export type Project = z.infer<typeof ProjectSchema>;

/**
 * Session schema
 */
export const SessionSchema = z.object({
  id: z.string(),
  project_id: z.string(),
  status: SessionStatusSchema,
  type: SessionTypeSchema,
  title: nullable(z.string()),
  created_at: DateStringSchema,
  updated_at: DateStringSchema,
  completed_at: nullable(DateStringSchema),
  agent_id: nullable(z.string()),
  pid: nullable(z.number()),
  metadata: z.record(z.unknown()).default({}),
});

export type Session = z.infer<typeof SessionSchema>;

/**
 * Session with token usage
 */
export const SessionWithUsageSchema = SessionSchema.extend({
  token_usage: nullable(TokenUsageSchema),
});

export type SessionWithUsage = z.infer<typeof SessionWithUsageSchema>;

/**
 * Create project params
 */
export const CreateProjectParamsSchema = z.object({
  name: z.string().min(1),
  path: FilePathSchema,
  description: z.string().optional(),
  tags: z.array(z.string()).optional(),
});

export type CreateProjectParams = z.infer<typeof CreateProjectParamsSchema>;

/**
 * Update project params
 */
export const UpdateProjectParamsSchema = z.object({
  name: z.string().min(1).optional(),
  description: z.string().optional(),
  tags: z.array(z.string()).optional(),
  metadata: z.record(z.unknown()).optional(),
});

export type UpdateProjectParams = z.infer<typeof UpdateProjectParamsSchema>;

/**
 * Project stats schema
 */
export const ProjectStatsSchema = z.object({
  total_sessions: z.number().int().nonnegative(),
  active_sessions: z.number().int().nonnegative(),
  total_tokens: z.number().int().nonnegative(),
  last_session_date: nullable(DateStringSchema),
  average_session_duration: nullable(z.number()),
  most_used_agent: nullable(z.string()),
});

export type ProjectStats = z.infer<typeof ProjectStatsSchema>;

/**
 * Project with stats schema
 */
export const ProjectWithStatsSchema = ProjectSchema.extend({
  stats: ProjectStatsSchema,
});

export type ProjectWithStats = z.infer<typeof ProjectWithStatsSchema>;

/**
 * Session create params
 */
export const CreateSessionParamsSchema = z.object({
  project_id: z.string(),
  type: SessionTypeSchema,
  title: z.string().optional(),
  agent_id: z.string().optional(),
  metadata: z.record(z.unknown()).optional(),
});

export type CreateSessionParams = z.infer<typeof CreateSessionParamsSchema>;

/**
 * Session list params
 */
export const ListSessionsParamsSchema = z.object({
  project_id: z.string().optional(),
  status: SessionStatusSchema.optional(),
  type: SessionTypeSchema.optional(),
  limit: z.number().int().positive().default(50),
  offset: z.number().int().nonnegative().default(0),
});

export type ListSessionsParams = z.infer<typeof ListSessionsParamsSchema>;

/**
 * Transform functions
 */
export const transformProject = (data: unknown): Project => {
  // Handle potential date format issues
  if (typeof data === 'object' && data !== null) {
    const obj = data as Record<string, unknown>;
    
    // Ensure dates are in ISO format
    if (obj.created_at && typeof obj.created_at === 'string') {
      obj.created_at = new Date(obj.created_at).toISOString();
    }
    if (obj.last_accessed && typeof obj.last_accessed === 'string') {
      obj.last_accessed = new Date(obj.last_accessed).toISOString();
    }
    
    // Ensure arrays
    if (!Array.isArray(obj.tags)) {
      obj.tags = [];
    }
    
    // Ensure metadata object
    if (!obj.metadata || typeof obj.metadata !== 'object') {
      obj.metadata = {};
    }
  }
  
  return ProjectSchema.parse(data);
};

export const transformSession = (data: unknown): Session => {
  if (typeof data === 'object' && data !== null) {
    const obj = data as Record<string, unknown>;
    
    // Ensure dates are in ISO format
    if (obj.created_at && typeof obj.created_at === 'string') {
      obj.created_at = new Date(obj.created_at).toISOString();
    }
    if (obj.updated_at && typeof obj.updated_at === 'string') {
      obj.updated_at = new Date(obj.updated_at).toISOString();
    }
    if (obj.completed_at && typeof obj.completed_at === 'string') {
      obj.completed_at = new Date(obj.completed_at).toISOString();
    }
    
    // Ensure metadata object
    if (!obj.metadata || typeof obj.metadata !== 'object') {
      obj.metadata = {};
    }
  }
  
  return SessionSchema.parse(data);
};