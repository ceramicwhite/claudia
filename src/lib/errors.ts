/**
 * Custom error classes for type-safe error handling
 */

export enum ErrorCode {
  // General errors
  UNKNOWN = 'UNKNOWN',
  VALIDATION = 'VALIDATION',
  NOT_FOUND = 'NOT_FOUND',
  UNAUTHORIZED = 'UNAUTHORIZED',
  FORBIDDEN = 'FORBIDDEN',
  
  // Tauri specific
  TAURI_INVOKE = 'TAURI_INVOKE',
  TAURI_EVENT = 'TAURI_EVENT',
  
  // Claude specific
  CLAUDE_PROCESS = 'CLAUDE_PROCESS',
  CLAUDE_SESSION = 'CLAUDE_SESSION',
  CLAUDE_NOT_INSTALLED = 'CLAUDE_NOT_INSTALLED',
  
  // Agent specific
  AGENT_EXECUTION = 'AGENT_EXECUTION',
  AGENT_NOT_FOUND = 'AGENT_NOT_FOUND',
  AGENT_INVALID_CONFIG = 'AGENT_INVALID_CONFIG',
  
  // MCP specific
  MCP_SERVER_ERROR = 'MCP_SERVER_ERROR',
  MCP_CONNECTION = 'MCP_CONNECTION',
  
  // Database specific
  DATABASE_ERROR = 'DATABASE_ERROR',
  DATABASE_MIGRATION = 'DATABASE_MIGRATION',
  
  // File system specific
  FILE_NOT_FOUND = 'FILE_NOT_FOUND',
  FILE_PERMISSION = 'FILE_PERMISSION',
  
  // Network specific
  NETWORK_ERROR = 'NETWORK_ERROR',
  TIMEOUT = 'TIMEOUT',
}

export interface ErrorDetails {
  code: ErrorCode;
  message: string;
  details?: unknown;
  timestamp: string;
  stackTrace?: string;
}

/**
 * Base application error class
 */
export class AppError extends Error {
  public readonly code: ErrorCode;
  public readonly details?: unknown;
  public readonly timestamp: string;

  constructor(code: ErrorCode, message: string, details?: unknown) {
    super(message);
    this.name = this.constructor.name;
    this.code = code;
    this.details = details;
    this.timestamp = new Date().toISOString();
    
    // Maintains proper stack trace for where our error was thrown
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, this.constructor);
    }
  }

  toJSON(): ErrorDetails {
    return {
      code: this.code,
      message: this.message,
      details: this.details,
      timestamp: this.timestamp,
      stackTrace: this.stack,
    };
  }

  static from(error: unknown): AppError {
    if (error instanceof AppError) {
      return error;
    }
    
    if (error instanceof Error) {
      return new AppError(
        ErrorCode.UNKNOWN,
        error.message,
        { originalError: error.name }
      );
    }
    
    return new AppError(
      ErrorCode.UNKNOWN,
      'An unknown error occurred',
      { originalError: error }
    );
  }
}

/**
 * Validation error for Zod schema failures
 */
export class ValidationError extends AppError {
  constructor(message: string, details?: unknown) {
    super(ErrorCode.VALIDATION, message, details);
  }
}

/**
 * Tauri invoke command error
 */
export class TauriError extends AppError {
  constructor(message: string, command?: string, details?: unknown) {
    super(ErrorCode.TAURI_INVOKE, message, { command, ...details });
  }
}

/**
 * Claude process related error
 */
export class ClaudeError extends AppError {
  constructor(code: ErrorCode, message: string, details?: unknown) {
    super(code, message, details);
  }
}

/**
 * Agent execution error
 */
export class AgentError extends AppError {
  constructor(code: ErrorCode, message: string, agentId?: string, details?: unknown) {
    super(code, message, { agentId, ...details });
  }
}

/**
 * MCP server error
 */
export class McpError extends AppError {
  constructor(message: string, serverId?: string, details?: unknown) {
    super(ErrorCode.MCP_SERVER_ERROR, message, { serverId, ...details });
  }
}

/**
 * Database error
 */
export class DatabaseError extends AppError {
  constructor(message: string, query?: string, details?: unknown) {
    const errorDetails = typeof details === 'object' && details !== null
      ? { query, ...details }
      : { query, details };
    super(ErrorCode.DATABASE_ERROR, message, errorDetails);
  }
}

/**
 * Type guard to check if error has a specific code
 */
export function hasErrorCode(error: unknown, code: ErrorCode): boolean {
  return error instanceof AppError && error.code === code;
}

/**
 * Error handler utility
 */
export class ErrorHandler {
  static handle(error: unknown): AppError {
    const appError = AppError.from(error);
    
    // Log error for debugging
    console.error('[ErrorHandler]', appError.toJSON());
    
    return appError;
  }
  
  static isRetryable(error: AppError): boolean {
    const retryableCodes = [
      ErrorCode.NETWORK_ERROR,
      ErrorCode.TIMEOUT,
      ErrorCode.MCP_CONNECTION,
    ];
    
    return retryableCodes.includes(error.code);
  }
  
  static getUserMessage(error: AppError): string {
    // Map technical errors to user-friendly messages
    const userMessages: Partial<Record<ErrorCode, string>> = {
      [ErrorCode.CLAUDE_NOT_INSTALLED]: 'Claude Code CLI is not installed. Please install it first.',
      [ErrorCode.NETWORK_ERROR]: 'Network connection failed. Please check your internet connection.',
      [ErrorCode.TIMEOUT]: 'The operation timed out. Please try again.',
      [ErrorCode.UNAUTHORIZED]: 'You are not authorized to perform this action.',
      [ErrorCode.DATABASE_ERROR]: 'A database error occurred. Please restart the application.',
    };
    
    return userMessages[error.code] || error.message;
  }
}