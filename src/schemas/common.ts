/**
 * Common Zod schemas used across the application
 */

import { z } from 'zod';

/**
 * ISO 8601 date string schema
 */
export const DateStringSchema = z.string().datetime();

/**
 * Unix timestamp schema
 */
export const TimestampSchema = z.number().int().positive();

/**
 * UUID schema
 */
export const UuidSchema = z.string().uuid();

/**
 * Non-empty string schema
 */
export const NonEmptyStringSchema = z.string().min(1);

/**
 * Email schema
 */
export const EmailSchema = z.string().email();

/**
 * URL schema
 */
export const UrlSchema = z.string().url();

/**
 * File path schema
 */
export const FilePathSchema = z.string().min(1);

/**
 * Nullable wrapper
 */
export function nullable<T extends z.ZodTypeAny>(schema: T) {
  return z.union([schema, z.null()]);
}

/**
 * Optional wrapper with default
 */
export function optionalWithDefault<T extends z.ZodTypeAny>(
  schema: T,
  defaultValue: z.infer<T>
) {
  return schema.optional().default(defaultValue);
}

/**
 * Pagination params schema
 */
export const PaginationParamsSchema = z.object({
  page: z.number().int().positive().default(1),
  limit: z.number().int().positive().max(100).default(20),
});

/**
 * Sort params schema
 */
export const SortParamsSchema = z.object({
  field: z.string(),
  order: z.enum(['asc', 'desc']).default('asc'),
});

/**
 * Common response wrapper schema
 */
export function createResponseSchema<T extends z.ZodTypeAny>(dataSchema: T) {
  return z.object({
    success: z.boolean(),
    data: dataSchema,
    error: z.string().optional(),
    timestamp: DateStringSchema.optional(),
  });
}

/**
 * List response schema
 */
export function createListResponseSchema<T extends z.ZodTypeAny>(itemSchema: T) {
  return z.object({
    items: z.array(itemSchema),
    total: z.number().int().nonnegative(),
    page: z.number().int().positive(),
    limit: z.number().int().positive(),
    hasMore: z.boolean(),
  });
}

/**
 * Token usage schema
 */
export const TokenUsageSchema = z.object({
  input_tokens: z.number().int().nonnegative(),
  output_tokens: z.number().int().nonnegative(),
  cache_creation_input_tokens: z.number().int().nonnegative().optional(),
  cache_read_input_tokens: z.number().int().nonnegative().optional(),
  total_tokens: z.number().int().nonnegative().optional(),
});

/**
 * Key-value pair schema
 */
export const KeyValueSchema = z.object({
  key: z.string(),
  value: z.unknown(),
});

/**
 * Error response schema
 */
export const ErrorResponseSchema = z.object({
  code: z.string(),
  message: z.string(),
  details: z.unknown().optional(),
  timestamp: DateStringSchema,
  stackTrace: z.string().optional(),
});

/**
 * Transform to ensure arrays
 */
export function ensureArray<T>(value: T | T[]): T[] {
  return Array.isArray(value) ? value : [value];
}

/**
 * Transform to parse JSON strings
 */
export function parseJsonString<T>(value: string | T): T {
  if (typeof value === 'string') {
    try {
      return JSON.parse(value);
    } catch {
      return value as T;
    }
  }
  return value;
}

/**
 * Safe enum schema that handles unknown values
 */
export function createSafeEnumSchema<T extends string>(
  values: readonly T[],
  defaultValue: T
) {
  return z.union([
    z.enum(values as [T, ...T[]]),
    z.string().transform(() => defaultValue),
  ]);
}