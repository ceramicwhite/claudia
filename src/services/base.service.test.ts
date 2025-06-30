import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { z } from 'zod';

// Mock the tauri invoke function before importing anything that uses it
const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: any[]) => mockInvoke(...args),
}));

import { BaseService, ServiceConfig } from './base.service';
import { AppError, TauriError, ValidationError, ErrorCode, ErrorHandler } from '@/lib/errors';

// Mock console methods to verify logging
const originalConsoleLog = console.log;
const originalConsoleError = console.error;
const mockConsoleLog = vi.fn();
const mockConsoleError = vi.fn();

// Test implementation of BaseService
class TestService extends BaseService {
  constructor(config: ServiceConfig) {
    super(config);
  }

  // Expose protected methods for testing
  async testInvoke<TInput, TOutput>(
    command: string,
    args: TInput,
    schema: z.ZodSchema<TOutput>
  ): Promise<TOutput> {
    return this.invoke(command, args, schema);
  }

  async testInvokeVoid<TInput>(command: string, args: TInput): Promise<void> {
    return this.invokeVoid(command, args);
  }

  testCreateTransformer<TInput, TOutput>(
    inputSchema: z.ZodSchema<TInput>,
    transform: (input: TInput) => TOutput
  ): z.ZodSchema<TOutput> {
    return this.createTransformer(inputSchema, transform);
  }

  async testBatchInvoke<T extends Record<string, unknown>>(
    commands: {
      [K in keyof T]: {
        command: string;
        args: unknown;
        schema: z.ZodSchema<T[K]>;
      };
    }
  ): Promise<T> {
    return this.batchInvoke(commands);
  }

  testLog(message: string, data?: unknown): void {
    return this.log(message, data);
  }

  testLogError(message: string, error: AppError): void {
    return this.logError(message, error);
  }

  testCreateCachedMethod<TArgs extends unknown[], TResult>(
    method: (...args: TArgs) => Promise<TResult>,
    getCacheKey: (...args: TArgs) => string,
    ttl?: number
  ): (...args: TArgs) => Promise<TResult> {
    return this.createCachedMethod(method, getCacheKey, ttl);
  }
}

describe('BaseService', () => {
  let service: TestService;

  beforeEach(() => {
    vi.clearAllMocks();
    console.log = mockConsoleLog;
    console.error = mockConsoleError;
    
    service = new TestService({
      serviceName: 'TestService',
      enableLogging: true,
    });
  });

  afterEach(() => {
    console.log = originalConsoleLog;
    console.error = originalConsoleError;
  });

  describe('constructor', () => {
    it('should initialize with provided config', () => {
      const config: ServiceConfig = {
        serviceName: 'CustomService',
        enableLogging: false,
        retryConfig: {
          maxRetries: 3,
          retryDelay: 1000,
          retryableErrors: ['NETWORK_ERROR'],
        },
      };

      const customService = new TestService(config);
      expect(customService['serviceName']).toBe('CustomService');
      expect(customService['enableLogging']).toBe(false);
      expect(customService['retryConfig']).toEqual(config.retryConfig);
    });

    it('should use default values when not provided', () => {
      const minimalService = new TestService({
        serviceName: 'MinimalService',
      });

      expect(minimalService['serviceName']).toBe('MinimalService');
      expect(minimalService['enableLogging']).toBe(true);
      expect(minimalService['retryConfig']).toBeUndefined();
    });
  });

  describe('invoke', () => {
    const testSchema = z.object({
      id: z.number(),
      name: z.string(),
    });

    it('should successfully invoke command and validate response', async () => {
      const mockResponse = { id: 1, name: 'Test' };
      mockInvoke.mockResolvedValueOnce(mockResponse);

      const result = await service.testInvoke('test_command', { test: true }, testSchema);

      expect(mockInvoke).toHaveBeenCalledWith('test_command', { test: true });
      expect(result).toEqual(mockResponse);
      expect(mockConsoleLog).toHaveBeenCalledWith(
        '[TestService] Invoking command: test_command',
        { test: true }
      );
      expect(mockConsoleLog).toHaveBeenCalledWith(
        '[TestService] Command test_command completed successfully',
        ''
      );
    });

    it('should throw ValidationError on schema validation failure', async () => {
      const invalidResponse = { id: 'not-a-number', name: 'Test' };
      mockInvoke.mockResolvedValueOnce(invalidResponse);

      try {
        await service.testInvoke('test_command', {}, testSchema);
        expect.fail('Should have thrown ValidationError');
      } catch (error: any) {
        expect(error).toBeInstanceOf(ValidationError);
        expect(error.message).toBe('Response validation failed');
        expect(error.details).toHaveProperty('errors');
        expect(error.details.received).toEqual(invalidResponse);
      }
    });

    it('should handle and transform Tauri errors', async () => {
      const originalError = new Error('Tauri error');
      mockInvoke.mockRejectedValueOnce(originalError);

      try {
        await service.testInvoke('test_command', {}, testSchema);
        expect.fail('Should have thrown TauriError');
      } catch (error: any) {
        expect(error).toBeInstanceOf(TauriError);
        expect(error.message).toBe('Tauri error');
        expect(error.details).toHaveProperty('command', 'test_command');
        expect(mockConsoleError).toHaveBeenCalled();
      }
    });

    it('should handle non-Error objects', async () => {
      mockInvoke.mockRejectedValueOnce('string error');

      try {
        await service.testInvoke('test_command', {}, testSchema);
        expect.fail('Should have thrown TauriError');
      } catch (error: any) {
        expect(error).toBeInstanceOf(TauriError);
        expect(error.message).toBe('Unknown error occurred');
      }
    });
  });

  describe('invokeVoid', () => {
    it('should successfully invoke command without validation', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await service.testInvokeVoid('void_command', { data: 'test' });

      expect(mockInvoke).toHaveBeenCalledWith('void_command', { data: 'test' });
      expect(mockConsoleLog).toHaveBeenCalledWith(
        '[TestService] Invoking command: void_command',
        { data: 'test' }
      );
      expect(mockConsoleLog).toHaveBeenCalledWith(
        '[TestService] Command void_command completed successfully',
        ''
      );
    });

    it('should handle errors in void commands', async () => {
      const error = new Error('Void command failed');
      mockInvoke.mockRejectedValueOnce(error);

      await expect(
        service.testInvokeVoid('void_command', {})
      ).rejects.toThrow(TauriError);
    });
  });

  describe('retry logic', () => {
    const retrySchema = z.string();

    beforeEach(() => {
      vi.useFakeTimers();
      service = new TestService({
        serviceName: 'RetryService',
        enableLogging: true,
        retryConfig: {
          maxRetries: 3,
          retryDelay: 1000,
        },
      });
    });

    afterEach(() => {
      vi.useRealTimers();
    });

    it('should retry on retryable errors', async () => {
      const networkError = new AppError(ErrorCode.NETWORK_ERROR, 'Network failed');
      
      mockInvoke
        .mockRejectedValueOnce(networkError)
        .mockRejectedValueOnce(networkError)
        .mockResolvedValueOnce('success');

      // Mock ErrorHandler.isRetryable
      vi.spyOn(ErrorHandler, 'isRetryable').mockReturnValue(true);

      const promise = service.testInvoke('retry_command', {}, retrySchema);

      // Fast-forward through retry delays
      await vi.runAllTimersAsync();

      const result = await promise;
      expect(result).toBe('success');
      expect(mockInvoke).toHaveBeenCalledTimes(3);
      expect(mockConsoleLog).toHaveBeenCalledWith(
        '[RetryService] Retrying command retry_command (attempt 1/3)',
        ''
      );
    });

    it('should use exponential backoff for retries', async () => {
      const networkError = new AppError(ErrorCode.NETWORK_ERROR, 'Network failed');
      
      mockInvoke
        .mockRejectedValueOnce(networkError)
        .mockRejectedValueOnce(networkError)
        .mockRejectedValueOnce(networkError)
        .mockResolvedValueOnce('success');

      vi.spyOn(ErrorHandler, 'isRetryable').mockReturnValue(true);

      const delayPromises: number[] = [];
      const originalSetTimeout = global.setTimeout;
      global.setTimeout = vi.fn((fn: any, delay?: number) => {
        if (delay) delayPromises.push(delay);
        return originalSetTimeout(fn, delay);
      }) as any;

      const promise = service.testInvoke('retry_command', {}, retrySchema);
      await vi.runAllTimersAsync();
      await promise;

      // Verify exponential backoff: 1000, 2000, 4000
      expect(delayPromises).toEqual([1000, 2000, 4000]);

      global.setTimeout = originalSetTimeout;
    });

    it('should not retry non-retryable errors', async () => {
      const validationError = new AppError(ErrorCode.VALIDATION, 'Validation failed');
      
      mockInvoke.mockRejectedValueOnce(validationError);
      vi.spyOn(ErrorHandler, 'isRetryable').mockReturnValue(false);

      await expect(
        service.testInvoke('no_retry_command', {}, retrySchema)
      ).rejects.toThrow();

      expect(mockInvoke).toHaveBeenCalledTimes(1);
    });

    it('should throw last error after max retries', async () => {
      const networkError = new AppError(ErrorCode.NETWORK_ERROR, 'Network failed');
      
      mockInvoke.mockRejectedValue(networkError);
      vi.spyOn(ErrorHandler, 'isRetryable').mockReturnValue(true);

      const promise = service.testInvoke('max_retry_command', {}, retrySchema);
      await vi.runAllTimersAsync();

      await expect(promise).rejects.toThrow(AppError);
      await expect(promise).rejects.toMatchObject({
        code: ErrorCode.NETWORK_ERROR,
        message: 'Network failed'
      });
      expect(mockInvoke).toHaveBeenCalledTimes(4); // 1 initial + 3 retries
    });
  });

  describe('createTransformer', () => {
    it('should create a schema transformer', () => {
      const inputSchema = z.number();
      const transformer = service.testCreateTransformer(
        inputSchema,
        (num: number) => num.toString()
      );

      const result = transformer.parse(42);
      expect(result).toBe('42');
    });

    it('should validate input before transformation', () => {
      const inputSchema = z.number();
      const transformer = service.testCreateTransformer(
        inputSchema,
        (num: number) => num.toString()
      );

      expect(() => transformer.parse('not-a-number')).toThrow(z.ZodError);
    });
  });

  describe('batchInvoke', () => {
    it('should execute multiple commands in parallel', async () => {
      mockInvoke
        .mockResolvedValueOnce({ id: 1 })
        .mockResolvedValueOnce('result2')
        .mockResolvedValueOnce(true);

      const commands = {
        first: {
          command: 'cmd1',
          args: { arg: 1 },
          schema: z.object({ id: z.number() }),
        },
        second: {
          command: 'cmd2',
          args: { arg: 2 },
          schema: z.string(),
        },
        third: {
          command: 'cmd3',
          args: { arg: 3 },
          schema: z.boolean(),
        },
      };

      const results = await service.testBatchInvoke(commands);

      expect(results).toEqual({
        first: { id: 1 },
        second: 'result2',
        third: true,
      });
      expect(mockInvoke).toHaveBeenCalledTimes(3);
    });

    it('should handle errors in batch operations', async () => {
      mockInvoke
        .mockResolvedValueOnce({ id: 1 })
        .mockRejectedValueOnce(new Error('Command 2 failed'))
        .mockResolvedValueOnce(true);

      const commands = {
        first: {
          command: 'cmd1',
          args: {},
          schema: z.object({ id: z.number() }),
        },
        second: {
          command: 'cmd2',
          args: {},
          schema: z.string(),
        },
        third: {
          command: 'cmd3',
          args: {},
          schema: z.boolean(),
        },
      };

      await expect(service.testBatchInvoke(commands)).rejects.toThrow(TauriError);
    });
  });

  describe('logging', () => {
    it('should log when logging is enabled', () => {
      service.testLog('Test message', { data: 'test' });

      expect(mockConsoleLog).toHaveBeenCalledWith(
        '[TestService] Test message',
        { data: 'test' }
      );
    });

    it('should not log when logging is disabled', () => {
      const silentService = new TestService({
        serviceName: 'SilentService',
        enableLogging: false,
      });

      silentService.testLog('Should not log');

      expect(mockConsoleLog).not.toHaveBeenCalled();
    });

    it('should handle logging without data', () => {
      service.testLog('Message without data');

      expect(mockConsoleLog).toHaveBeenCalledWith(
        '[TestService] Message without data',
        ''
      );
    });

    it('should always log errors regardless of enableLogging', () => {
      const error = new AppError(ErrorCode.UNKNOWN, 'Test error');
      
      service.testLogError('Error occurred', error);

      expect(mockConsoleError).toHaveBeenCalledWith(
        '[TestService] Error occurred',
        error.toJSON()
      );

      // Test with logging disabled
      const silentService = new TestService({
        serviceName: 'SilentService',
        enableLogging: false,
      });

      mockConsoleError.mockClear();
      silentService.testLogError('Error in silent service', error);

      expect(mockConsoleError).toHaveBeenCalledWith(
        '[SilentService] Error in silent service',
        error.toJSON()
      );
    });
  });

  describe('createCachedMethod', () => {
    it('should cache method results', async () => {
      let callCount = 0;
      const expensiveMethod = async (id: number): Promise<string> => {
        callCount++;
        return `result-${id}`;
      };

      const cachedMethod = service.testCreateCachedMethod(
        expensiveMethod,
        (id) => `key-${id}`,
        1000
      );

      // First call should execute the method
      const result1 = await cachedMethod(1);
      expect(result1).toBe('result-1');
      expect(callCount).toBe(1);

      // Second call with same argument should use cache
      const result2 = await cachedMethod(1);
      expect(result2).toBe('result-1');
      expect(callCount).toBe(1);

      // Call with different argument should execute the method
      const result3 = await cachedMethod(2);
      expect(result3).toBe('result-2');
      expect(callCount).toBe(2);

      expect(mockConsoleLog).toHaveBeenCalledWith('[TestService] Cache hit for key: key-1', '');
    });

    it('should expire cache after TTL', async () => {
      vi.useFakeTimers();
      
      let callCount = 0;
      const method = async (id: number): Promise<string> => {
        callCount++;
        return `result-${id}-${callCount}`;
      };

      const cachedMethod = service.testCreateCachedMethod(
        method,
        (id) => `key-${id}`,
        1000 // 1 second TTL
      );

      // First call
      const result1 = await cachedMethod(1);
      expect(result1).toBe('result-1-1');
      expect(callCount).toBe(1);

      // Advance time by 500ms (within TTL)
      vi.advanceTimersByTime(500);
      const result2 = await cachedMethod(1);
      expect(result2).toBe('result-1-1');
      expect(callCount).toBe(1);

      // Advance time by another 600ms (beyond TTL)
      vi.advanceTimersByTime(600);
      const result3 = await cachedMethod(1);
      expect(result3).toBe('result-1-2');
      expect(callCount).toBe(2);

      vi.useRealTimers();
    });

    it('should clean up expired entries', async () => {
      vi.useFakeTimers();
      
      const method = async (id: number): Promise<string> => `result-${id}`;
      const cachedMethod = service.testCreateCachedMethod(
        method,
        (id) => `key-${id}`,
        1000
      );

      // Create multiple cache entries with specific timing
      await cachedMethod(1); // t=0, expires at t=1000
      vi.advanceTimersByTime(500); // t=500
      await cachedMethod(2); // t=500, expires at t=1500
      vi.advanceTimersByTime(600); // t=1100
      await cachedMethod(3); // t=1100, expires at t=2100, should trigger cleanup of entry 1

      // Now entry 1 is expired (t=1100 > 1000)
      // Entry 2 expires at t=1500
      // Entry 3 expires at t=2100
      
      // Clear previous calls to check for the specific cache hit
      mockConsoleLog.mockClear();
      
      // Don't advance time, just try to get entry 2 which should still be valid
      await cachedMethod(2); // Should hit cache since t=1100 < 1500
      expect(mockConsoleLog).toHaveBeenCalledWith('[TestService] Cache hit for key: key-2', '');

      vi.useRealTimers();
    });

    it('should use default TTL when not specified', async () => {
      const method = async (): Promise<string> => 'result';
      const cachedMethod = service.testCreateCachedMethod(
        method,
        () => 'default-key'
      );

      await cachedMethod();
      await cachedMethod();

      expect(mockConsoleLog).toHaveBeenCalledWith('[TestService] Cache hit for key: default-key', '');
    });
  });

  describe('error handling edge cases', () => {
    it('should handle circular references in error details', async () => {
      const circularObj: any = { a: 1 };
      circularObj.self = circularObj;
      
      const error = new AppError(ErrorCode.UNKNOWN, 'Circular error', circularObj);
      mockInvoke.mockRejectedValueOnce(error);

      await expect(
        service.testInvoke('circular_command', {}, z.any())
      ).rejects.toThrow();

      // Should not throw when logging the error
      expect(mockConsoleError).toHaveBeenCalled();
    });

    it('should preserve stack traces', async () => {
      const originalError = new Error('Original error');
      mockInvoke.mockRejectedValueOnce(originalError);

      try {
        await service.testInvoke('stack_command', {}, z.any());
      } catch (error: any) {
        expect(error.stack).toBeDefined();
        expect(error).toBeInstanceOf(TauriError);
      }
    });
  });

  describe('concurrent operations', () => {
    it('should handle concurrent invocations', async () => {
      mockInvoke
        .mockImplementation((command) => {
          return Promise.resolve(`${command}-result`);
        });

      const promises = [
        service.testInvoke('cmd1', {}, z.string()),
        service.testInvoke('cmd2', {}, z.string()),
        service.testInvoke('cmd3', {}, z.string()),
      ];

      const results = await Promise.all(promises);
      expect(results).toEqual(['cmd1-result', 'cmd2-result', 'cmd3-result']);
      expect(mockInvoke).toHaveBeenCalledTimes(3);
    });
  });

  describe('type transformations', () => {
    it('should handle complex schema transformations', () => {
      const dateSchema = z.string().transform(str => new Date(str));
      const objectSchema = z.object({
        id: z.number(),
        createdAt: dateSchema,
        tags: z.array(z.string()).transform(tags => new Set(tags)),
      });

      const input = {
        id: 1,
        createdAt: '2024-01-01T00:00:00Z',
        tags: ['tag1', 'tag2', 'tag1'],
      };

      const result = objectSchema.parse(input);
      expect(result.id).toBe(1);
      expect(result.createdAt).toBeInstanceOf(Date);
      expect(result.tags).toBeInstanceOf(Set);
      expect(result.tags.size).toBe(2);
    });
  });
});