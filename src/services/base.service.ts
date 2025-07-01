/**
 * Base service class with type-safe Tauri command invocation and error handling
 */

import { invoke } from '@tauri-apps/api/core';
import { z } from 'zod';
import { AppError, TauriError, ValidationError, ErrorHandler } from '@/lib/errors';

export interface ServiceConfig {
  serviceName: string;
  enableLogging?: boolean;
  retryConfig?: {
    maxRetries: number;
    retryDelay: number;
    retryableErrors?: string[];
  };
}

export abstract class BaseService {
  protected readonly serviceName: string;
  protected readonly enableLogging: boolean;
  protected readonly retryConfig?: ServiceConfig['retryConfig'];

  constructor(config: ServiceConfig) {
    this.serviceName = config.serviceName;
    this.enableLogging = config.enableLogging ?? true;
    this.retryConfig = config.retryConfig;
  }

  /**
   * Invoke a Tauri command with type-safe validation
   */
  protected async invoke<TInput, TOutput>(
    command: string,
    args: TInput,
    schemaOrErrorMessage?: z.ZodSchema<TOutput> | string
  ): Promise<TOutput> {
    try {
      this.log(`Invoking command: ${command}`, args);
      
      const result = await this.invokeWithRetry(command, args);
      
      // If a schema is provided, validate the response
      if (schemaOrErrorMessage && typeof schemaOrErrorMessage !== 'string') {
        const validated = await this.validateResponse(result, schemaOrErrorMessage);
        this.log(`Command ${command} completed successfully`);
        return validated;
      }
      
      this.log(`Command ${command} completed successfully`);
      return result as TOutput;
    } catch (error) {
      return this.handleError(error, command);
    }
  }

  /**
   * Invoke a Tauri command without validation (for void responses)
   */
  protected async invokeVoid<TInput>(
    command: string,
    args: TInput
  ): Promise<void> {
    try {
      this.log(`Invoking command: ${command}`, args);
      
      await this.invokeWithRetry(command, args);
      
      this.log(`Command ${command} completed successfully`);
    } catch (error) {
      this.handleError(error, command);
    }
  }

  /**
   * Invoke a command that takes no arguments
   */
  protected async invokeNoArgs<TOutput>(
    command: string,
    schemaOrErrorMessage?: z.ZodSchema<TOutput> | string
  ): Promise<TOutput> {
    return this.invoke(command, {}, schemaOrErrorMessage);
  }

  /**
   * Invoke with retry logic
   */
  private async invokeWithRetry<TInput>(
    command: string,
    args: TInput
  ): Promise<unknown> {
    let lastError: unknown;
    const maxRetries = this.retryConfig?.maxRetries ?? 0;
    const retryDelay = this.retryConfig?.retryDelay ?? 1000;
    
    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      try {
        return await invoke(command, args as any);
      } catch (error) {
        lastError = error;
        
        const appError = AppError.from(error);
        if (
          attempt < maxRetries &&
          ErrorHandler.isRetryable(appError)
        ) {
          this.log(`Retrying command ${command} (attempt ${attempt + 1}/${maxRetries})`);
          await this.delay(retryDelay * Math.pow(2, attempt)); // Exponential backoff
        } else {
          break;
        }
      }
    }
    
    throw lastError;
  }

  /**
   * Validate response against schema
   */
  private async validateResponse<T>(
    data: unknown,
    schema: z.ZodSchema<T>
  ): Promise<T> {
    try {
      return schema.parse(data);
    } catch (error) {
      if (error instanceof z.ZodError) {
        throw new ValidationError(
          'Response validation failed',
          {
            errors: error.errors,
            received: data,
          }
        );
      }
      throw error;
    }
  }

  /**
   * Handle errors consistently
   */
  private handleError(error: unknown, command: string): never {
    const appError = error instanceof AppError
      ? error
      : error instanceof Error
      ? new TauriError(error.message, command, { originalError: error })
      : new TauriError('Unknown error occurred', command, { originalError: error });
    
    this.logError(`Command ${command} failed`, appError);
    throw appError;
  }

  /**
   * Create a schema transformer for common transformations
   */
  protected createTransformer<TInput, TOutput>(
    inputSchema: z.ZodSchema<TInput>,
    transform: (input: TInput) => TOutput
  ): z.ZodEffects<z.ZodSchema<TInput>, TOutput, TInput> {
    return inputSchema.transform(transform);
  }

  /**
   * Batch invoke multiple commands
   */
  protected async batchInvoke<T extends Record<string, unknown>>(
    commands: {
      [K in keyof T]: {
        command: string;
        args: unknown;
        schema: z.ZodSchema<T[K]>;
      };
    }
  ): Promise<T> {
    const entries = Object.entries(commands) as Array<[keyof T, typeof commands[keyof T]]>;
    const promises = entries.map(async ([key, config]) => {
      const result = await this.invoke(config.command, config.args, config.schema);
      return [key, result] as const;
    });
    
    const results = await Promise.all(promises);
    return Object.fromEntries(results) as T;
  }

  /**
   * Log a message if logging is enabled
   */
  protected log(message: string, data?: unknown): void {
    if (this.enableLogging) {
      console.log(`[${this.serviceName}] ${message}`, data ?? '');
    }
  }

  /**
   * Log an error
   */
  protected logError(message: string, error: AppError): void {
    console.error(`[${this.serviceName}] ${message}`, error.toJSON());
  }

  /**
   * Delay utility
   */
  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * Safe invoke that returns a default value on error
   */
  protected async safeInvoke<T>(
    promise: Promise<T>,
    defaultValue: T
  ): Promise<T> {
    try {
      return await promise;
    } catch (error) {
      this.logError('Safe invoke caught error, returning default', AppError.from(error));
      return defaultValue;
    }
  }

  /**
   * Create a cached method
   */
  protected createCachedMethod<TArgs extends unknown[], TResult>(
    method: (...args: TArgs) => Promise<TResult>,
    getCacheKey: (...args: TArgs) => string,
    ttl: number = 60000 // 1 minute default
  ): (...args: TArgs) => Promise<TResult> {
    const cache = new Map<string, { value: TResult; expires: number }>();
    
    return async (...args: TArgs): Promise<TResult> => {
      const key = getCacheKey(...args);
      const cached = cache.get(key);
      
      if (cached && cached.expires > Date.now()) {
        this.log(`Cache hit for key: ${key}`);
        return cached.value;
      }
      
      const result = await method.apply(this, args);
      cache.set(key, { value: result, expires: Date.now() + ttl });
      
      // Clean up expired entries
      for (const [k, v] of cache.entries()) {
        if (v.expires <= Date.now()) {
          cache.delete(k);
        }
      }
      
      return result;
    };
  }
}