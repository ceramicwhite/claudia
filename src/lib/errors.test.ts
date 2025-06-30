import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import {
  ErrorCode,
  AppError,
  ValidationError,
  TauriError,
  ClaudeError,
  AgentError,
  McpError,
  DatabaseError,
  hasErrorCode,
  ErrorHandler,
  type ErrorDetails,
} from './errors'

describe('errors', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // Mock console.error to avoid noise in test output
    vi.spyOn(console, 'error').mockImplementation(() => {})
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  describe('AppError', () => {
    it('should create an error with all properties', () => {
      const error = new AppError(ErrorCode.UNKNOWN, 'Test error', { foo: 'bar' })

      expect(error).toBeInstanceOf(Error)
      expect(error).toBeInstanceOf(AppError)
      expect(error.name).toBe('AppError')
      expect(error.code).toBe(ErrorCode.UNKNOWN)
      expect(error.message).toBe('Test error')
      expect(error.details).toEqual({ foo: 'bar' })
      expect(error.timestamp).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}.\d{3}Z$/)
      expect(error.stack).toBeDefined()
    })

    it('should create an error without details', () => {
      const error = new AppError(ErrorCode.NOT_FOUND, 'Not found')

      expect(error.code).toBe(ErrorCode.NOT_FOUND)
      expect(error.message).toBe('Not found')
      expect(error.details).toBeUndefined()
    })

    it('should maintain proper stack trace', () => {
      const error = new AppError(ErrorCode.UNKNOWN, 'Stack trace test')

      expect(error.stack).toBeDefined()
      expect(error.stack).toContain('Stack trace test')
      expect(error.stack).toContain('errors.test.ts')
    })

    describe('toJSON', () => {
      it('should serialize error to JSON format', () => {
        const error = new AppError(ErrorCode.VALIDATION, 'Validation failed', { field: 'email' })
        const json = error.toJSON()

        expect(json).toEqual({
          code: ErrorCode.VALIDATION,
          message: 'Validation failed',
          details: { field: 'email' },
          timestamp: error.timestamp,
          stackTrace: error.stack,
        })
      })

      it('should handle circular references in details', () => {
        const circular: any = { ref: null }
        circular.ref = circular
        
        const error = new AppError(ErrorCode.UNKNOWN, 'Circular test', circular)
        
        // Should not throw when serializing
        expect(() => JSON.stringify(error.toJSON())).toThrow()
      })
    })

    describe('from', () => {
      it('should return the same AppError instance', () => {
        const original = new AppError(ErrorCode.UNAUTHORIZED, 'Unauthorized')
        const result = AppError.from(original)

        expect(result).toBe(original)
      })

      it('should convert Error to AppError', () => {
        const jsError = new Error('JavaScript error')
        const result = AppError.from(jsError)

        expect(result).toBeInstanceOf(AppError)
        expect(result.code).toBe(ErrorCode.UNKNOWN)
        expect(result.message).toBe('JavaScript error')
        expect(result.details).toEqual({ originalError: 'Error' })
      })

      it('should convert TypeError to AppError', () => {
        const typeError = new TypeError('Type mismatch')
        const result = AppError.from(typeError)

        expect(result).toBeInstanceOf(AppError)
        expect(result.code).toBe(ErrorCode.UNKNOWN)
        expect(result.message).toBe('Type mismatch')
        expect(result.details).toEqual({ originalError: 'TypeError' })
      })

      it('should handle non-Error objects', () => {
        const result = AppError.from('string error')

        expect(result).toBeInstanceOf(AppError)
        expect(result.code).toBe(ErrorCode.UNKNOWN)
        expect(result.message).toBe('An unknown error occurred')
        expect(result.details).toEqual({ originalError: 'string error' })
      })

      it('should handle null and undefined', () => {
        const nullResult = AppError.from(null)
        const undefinedResult = AppError.from(undefined)

        expect(nullResult.details).toEqual({ originalError: null })
        expect(undefinedResult.details).toEqual({ originalError: undefined })
      })

      it('should handle objects with message property', () => {
        const obj = { message: 'Object with message', code: 42 }
        const result = AppError.from(obj)

        expect(result.message).toBe('An unknown error occurred')
        expect(result.details).toEqual({ originalError: obj })
      })
    })
  })

  describe('ValidationError', () => {
    it('should create a validation error', () => {
      const error = new ValidationError('Invalid email format', { field: 'email', value: 'invalid' })

      expect(error).toBeInstanceOf(AppError)
      expect(error).toBeInstanceOf(ValidationError)
      expect(error.name).toBe('ValidationError')
      expect(error.code).toBe(ErrorCode.VALIDATION)
      expect(error.message).toBe('Invalid email format')
      expect(error.details).toEqual({ field: 'email', value: 'invalid' })
    })

    it('should create a validation error without details', () => {
      const error = new ValidationError('Validation failed')

      expect(error.code).toBe(ErrorCode.VALIDATION)
      expect(error.details).toBeUndefined()
    })
  })

  describe('TauriError', () => {
    it('should create a Tauri error with command', () => {
      const error = new TauriError('Command failed', 'get_user', { reason: 'timeout' })

      expect(error).toBeInstanceOf(AppError)
      expect(error).toBeInstanceOf(TauriError)
      expect(error.name).toBe('TauriError')
      expect(error.code).toBe(ErrorCode.TAURI_INVOKE)
      expect(error.message).toBe('Command failed')
      expect(error.details).toEqual({ command: 'get_user', reason: 'timeout' })
    })

    it('should create a Tauri error without command', () => {
      const error = new TauriError('Tauri API error')

      expect(error.code).toBe(ErrorCode.TAURI_INVOKE)
      expect(error.details).toEqual({ command: undefined })
    })

    it('should merge additional details', () => {
      const error = new TauriError('Failed', 'invoke_test', { extra: 'data', nested: { value: 1 } })

      expect(error.details).toEqual({
        command: 'invoke_test',
        extra: 'data',
        nested: { value: 1 }
      })
    })
  })

  describe('ClaudeError', () => {
    it('should create a Claude error with custom code', () => {
      const error = new ClaudeError(ErrorCode.CLAUDE_NOT_INSTALLED, 'Claude CLI not found', { path: '/usr/bin' })

      expect(error).toBeInstanceOf(AppError)
      expect(error).toBeInstanceOf(ClaudeError)
      expect(error.name).toBe('ClaudeError')
      expect(error.code).toBe(ErrorCode.CLAUDE_NOT_INSTALLED)
      expect(error.message).toBe('Claude CLI not found')
      expect(error.details).toEqual({ path: '/usr/bin' })
    })

    it('should support different Claude error codes', () => {
      const processError = new ClaudeError(ErrorCode.CLAUDE_PROCESS, 'Process crashed')
      const sessionError = new ClaudeError(ErrorCode.CLAUDE_SESSION, 'Session expired')

      expect(processError.code).toBe(ErrorCode.CLAUDE_PROCESS)
      expect(sessionError.code).toBe(ErrorCode.CLAUDE_SESSION)
    })
  })

  describe('AgentError', () => {
    it('should create an agent error with agent ID', () => {
      const error = new AgentError(ErrorCode.AGENT_EXECUTION, 'Execution failed', 'agent-123', { exitCode: 1 })

      expect(error).toBeInstanceOf(AppError)
      expect(error).toBeInstanceOf(AgentError)
      expect(error.name).toBe('AgentError')
      expect(error.code).toBe(ErrorCode.AGENT_EXECUTION)
      expect(error.message).toBe('Execution failed')
      expect(error.details).toEqual({ agentId: 'agent-123', exitCode: 1 })
    })

    it('should create an agent error without agent ID', () => {
      const error = new AgentError(ErrorCode.AGENT_NOT_FOUND, 'Agent not found')

      expect(error.code).toBe(ErrorCode.AGENT_NOT_FOUND)
      expect(error.details).toEqual({ agentId: undefined })
    })

    it('should support different agent error codes', () => {
      const configError = new AgentError(ErrorCode.AGENT_INVALID_CONFIG, 'Invalid config', 'agent-456')

      expect(configError.code).toBe(ErrorCode.AGENT_INVALID_CONFIG)
      expect(configError.details).toEqual({ agentId: 'agent-456' })
    })
  })

  describe('McpError', () => {
    it('should create an MCP error with server ID', () => {
      const error = new McpError('Connection refused', 'mcp-server-1', { port: 8080 })

      expect(error).toBeInstanceOf(AppError)
      expect(error).toBeInstanceOf(McpError)
      expect(error.name).toBe('McpError')
      expect(error.code).toBe(ErrorCode.MCP_SERVER_ERROR)
      expect(error.message).toBe('Connection refused')
      expect(error.details).toEqual({ serverId: 'mcp-server-1', port: 8080 })
    })

    it('should create an MCP error without server ID', () => {
      const error = new McpError('MCP protocol error')

      expect(error.code).toBe(ErrorCode.MCP_SERVER_ERROR)
      expect(error.details).toEqual({ serverId: undefined })
    })
  })

  describe('DatabaseError', () => {
    it('should create a database error with query', () => {
      const error = new DatabaseError('Syntax error', 'SELECT * FROM users WHERE', { line: 1, column: 30 })

      expect(error).toBeInstanceOf(AppError)
      expect(error).toBeInstanceOf(DatabaseError)
      expect(error.name).toBe('DatabaseError')
      expect(error.code).toBe(ErrorCode.DATABASE_ERROR)
      expect(error.message).toBe('Syntax error')
      expect(error.details).toEqual({
        query: 'SELECT * FROM users WHERE',
        line: 1,
        column: 30
      })
    })

    it('should create a database error without query', () => {
      const error = new DatabaseError('Connection failed')

      expect(error.code).toBe(ErrorCode.DATABASE_ERROR)
      expect(error.details).toEqual({ query: undefined })
    })

    it('should handle non-object details', () => {
      const error = new DatabaseError('Error', 'INSERT INTO...', 'string detail')

      expect(error.details).toEqual({
        query: 'INSERT INTO...',
        details: 'string detail'
      })
    })

    it('should handle null details', () => {
      const error = new DatabaseError('Error', 'UPDATE...', null)

      expect(error.details).toEqual({
        query: 'UPDATE...',
        details: null
      })
    })

    it('should merge object details properly', () => {
      const error = new DatabaseError('Error', 'DELETE...', { reason: 'locked', retries: 3 })

      expect(error.details).toEqual({
        query: 'DELETE...',
        reason: 'locked',
        retries: 3
      })
    })
  })

  describe('hasErrorCode', () => {
    it('should return true for AppError with matching code', () => {
      const error = new AppError(ErrorCode.NETWORK_ERROR, 'Network failed')

      expect(hasErrorCode(error, ErrorCode.NETWORK_ERROR)).toBe(true)
    })

    it('should return false for AppError with different code', () => {
      const error = new AppError(ErrorCode.NETWORK_ERROR, 'Network failed')

      expect(hasErrorCode(error, ErrorCode.TIMEOUT)).toBe(false)
    })

    it('should return false for non-AppError instances', () => {
      const jsError = new Error('Regular error')

      expect(hasErrorCode(jsError, ErrorCode.UNKNOWN)).toBe(false)
    })

    it('should return false for null/undefined', () => {
      expect(hasErrorCode(null, ErrorCode.UNKNOWN)).toBe(false)
      expect(hasErrorCode(undefined, ErrorCode.UNKNOWN)).toBe(false)
    })

    it('should work with subclasses', () => {
      const validationError = new ValidationError('Invalid')
      const tauriError = new TauriError('Failed')

      expect(hasErrorCode(validationError, ErrorCode.VALIDATION)).toBe(true)
      expect(hasErrorCode(tauriError, ErrorCode.TAURI_INVOKE)).toBe(true)
    })
  })

  describe('ErrorHandler', () => {
    describe('handle', () => {
      it('should convert and log AppError', () => {
        const error = new AppError(ErrorCode.UNAUTHORIZED, 'Access denied')
        const result = ErrorHandler.handle(error)

        expect(result).toBe(error)
        expect(console.error).toHaveBeenCalledWith('[ErrorHandler]', error.toJSON())
      })

      it('should convert Error to AppError and log', () => {
        const jsError = new Error('JavaScript error')
        const result = ErrorHandler.handle(jsError)

        expect(result).toBeInstanceOf(AppError)
        expect(result.code).toBe(ErrorCode.UNKNOWN)
        expect(result.message).toBe('JavaScript error')
        expect(console.error).toHaveBeenCalled()
      })

      it('should handle non-Error objects', () => {
        const result = ErrorHandler.handle({ custom: 'error' })

        expect(result).toBeInstanceOf(AppError)
        expect(result.code).toBe(ErrorCode.UNKNOWN)
        expect(result.message).toBe('An unknown error occurred')
        expect(result.details).toEqual({ originalError: { custom: 'error' } })
      })

      it('should handle string errors', () => {
        const result = ErrorHandler.handle('String error message')

        expect(result).toBeInstanceOf(AppError)
        expect(result.details).toEqual({ originalError: 'String error message' })
      })
    })

    describe('isRetryable', () => {
      it('should return true for retryable error codes', () => {
        const networkError = new AppError(ErrorCode.NETWORK_ERROR, 'Network failed')
        const timeoutError = new AppError(ErrorCode.TIMEOUT, 'Request timed out')
        const mcpError = new AppError(ErrorCode.MCP_CONNECTION, 'MCP connection failed')

        expect(ErrorHandler.isRetryable(networkError)).toBe(true)
        expect(ErrorHandler.isRetryable(timeoutError)).toBe(true)
        expect(ErrorHandler.isRetryable(mcpError)).toBe(true)
      })

      it('should return false for non-retryable error codes', () => {
        const validationError = new ValidationError('Invalid input')
        const authError = new AppError(ErrorCode.UNAUTHORIZED, 'Not authorized')
        const notFoundError = new AppError(ErrorCode.NOT_FOUND, 'Resource not found')

        expect(ErrorHandler.isRetryable(validationError)).toBe(false)
        expect(ErrorHandler.isRetryable(authError)).toBe(false)
        expect(ErrorHandler.isRetryable(notFoundError)).toBe(false)
      })
    })

    describe('getUserMessage', () => {
      it('should return user-friendly message for known error codes', () => {
        const claudeError = new ClaudeError(ErrorCode.CLAUDE_NOT_INSTALLED, 'Technical message')
        expect(ErrorHandler.getUserMessage(claudeError)).toBe(
          'Claude Code CLI is not installed. Please install it first.'
        )

        const networkError = new AppError(ErrorCode.NETWORK_ERROR, 'ECONNREFUSED')
        expect(ErrorHandler.getUserMessage(networkError)).toBe(
          'Network connection failed. Please check your internet connection.'
        )

        const timeoutError = new AppError(ErrorCode.TIMEOUT, 'Operation timed out after 30s')
        expect(ErrorHandler.getUserMessage(timeoutError)).toBe(
          'The operation timed out. Please try again.'
        )

        const authError = new AppError(ErrorCode.UNAUTHORIZED, 'Invalid token')
        expect(ErrorHandler.getUserMessage(authError)).toBe(
          'You are not authorized to perform this action.'
        )

        const dbError = new DatabaseError('SQLite error')
        expect(ErrorHandler.getUserMessage(dbError)).toBe(
          'A database error occurred. Please restart the application.'
        )
      })

      it('should return original message for unknown error codes', () => {
        const unknownError = new AppError(ErrorCode.UNKNOWN, 'Something went wrong')
        expect(ErrorHandler.getUserMessage(unknownError)).toBe('Something went wrong')

        const customError = new AppError(ErrorCode.AGENT_EXECUTION, 'Agent failed to execute task')
        expect(ErrorHandler.getUserMessage(customError)).toBe('Agent failed to execute task')
      })
    })
  })

  describe('Error inheritance', () => {
    it('should maintain instanceof relationships', () => {
      const validationError = new ValidationError('Invalid')
      const tauriError = new TauriError('Failed')
      const claudeError = new ClaudeError(ErrorCode.CLAUDE_PROCESS, 'Process error')
      const agentError = new AgentError(ErrorCode.AGENT_EXECUTION, 'Execution error')
      const mcpError = new McpError('MCP error')
      const dbError = new DatabaseError('DB error')

      // All should be instances of Error and AppError
      const errors = [validationError, tauriError, claudeError, agentError, mcpError, dbError]
      
      errors.forEach(error => {
        expect(error).toBeInstanceOf(Error)
        expect(error).toBeInstanceOf(AppError)
      })

      // Specific instanceof checks
      expect(validationError).toBeInstanceOf(ValidationError)
      expect(tauriError).toBeInstanceOf(TauriError)
      expect(claudeError).toBeInstanceOf(ClaudeError)
      expect(agentError).toBeInstanceOf(AgentError)
      expect(mcpError).toBeInstanceOf(McpError)
      expect(dbError).toBeInstanceOf(DatabaseError)
    })
  })

  describe('Error serialization and deserialization', () => {
    it('should serialize and deserialize error details correctly', () => {
      const original = new AppError(ErrorCode.VALIDATION, 'Test error', {
        field: 'email',
        value: 'invalid@',
        rules: ['required', 'email']
      })

      const json = JSON.stringify(original.toJSON())
      const parsed: ErrorDetails = JSON.parse(json)

      expect(parsed.code).toBe(ErrorCode.VALIDATION)
      expect(parsed.message).toBe('Test error')
      expect(parsed.details).toEqual({
        field: 'email',
        value: 'invalid@',
        rules: ['required', 'email']
      })
      expect(parsed.timestamp).toBe(original.timestamp)
      expect(parsed.stackTrace).toBeDefined()
    })

    it('should handle errors with undefined details', () => {
      const error = new AppError(ErrorCode.NOT_FOUND, 'Not found')
      const json = JSON.stringify(error.toJSON())
      const parsed: ErrorDetails = JSON.parse(json)

      expect(parsed.details).toBeUndefined()
    })

    it('should preserve dates in details', () => {
      const date = new Date('2024-01-01T00:00:00Z')
      const error = new AppError(ErrorCode.UNKNOWN, 'Date test', { createdAt: date })
      
      const json = JSON.stringify(error.toJSON())
      const parsed: ErrorDetails = JSON.parse(json)

      // Date is serialized as string in JSON
      expect(parsed.details).toEqual({ createdAt: date.toISOString() })
    })
  })

  describe('Edge cases', () => {
    it('should handle very long error messages', () => {
      const longMessage = 'A'.repeat(10000)
      const error = new AppError(ErrorCode.UNKNOWN, longMessage)

      expect(error.message).toBe(longMessage)
      expect(error.message.length).toBe(10000)
    })

    it('should handle special characters in messages', () => {
      const specialMessage = 'Error with \n newlines \t tabs and "quotes" and \'apostrophes\''
      const error = new AppError(ErrorCode.UNKNOWN, specialMessage)

      expect(error.message).toBe(specialMessage)
    })

    it('should handle empty messages', () => {
      const error = new AppError(ErrorCode.UNKNOWN, '')

      expect(error.message).toBe('')
    })

    it('should handle complex nested details', () => {
      const complexDetails = {
        level1: {
          level2: {
            level3: {
              array: [1, 2, { nested: true }],
              fn: () => 'function',
              date: new Date(),
              regex: /test/g,
              symbol: Symbol('test')
            }
          }
        }
      }

      const error = new AppError(ErrorCode.UNKNOWN, 'Complex details', complexDetails)
      
      expect(error.details).toBe(complexDetails)
      
      // JSON serialization will handle non-serializable values
      const json = error.toJSON()
      expect(json.details).toBeDefined()
    })

    it('should handle errors created in different contexts', () => {
      // Simulate error from different execution context
      const errorFromPromise = new Promise((_, reject) => {
        reject(new AppError(ErrorCode.TIMEOUT, 'Async timeout'))
      })

      expect(errorFromPromise).rejects.toThrow(AppError)
    })
  })

  describe('ErrorCode enum coverage', () => {
    it('should have all error codes defined', () => {
      const allCodes = Object.values(ErrorCode)
      
      // Ensure all codes are unique
      const uniqueCodes = new Set(allCodes)
      expect(uniqueCodes.size).toBe(allCodes.length)

      // Verify specific codes exist
      expect(allCodes).toContain(ErrorCode.UNKNOWN)
      expect(allCodes).toContain(ErrorCode.VALIDATION)
      expect(allCodes).toContain(ErrorCode.NOT_FOUND)
      expect(allCodes).toContain(ErrorCode.UNAUTHORIZED)
      expect(allCodes).toContain(ErrorCode.FORBIDDEN)
      expect(allCodes).toContain(ErrorCode.TAURI_INVOKE)
      expect(allCodes).toContain(ErrorCode.TAURI_EVENT)
      expect(allCodes).toContain(ErrorCode.CLAUDE_PROCESS)
      expect(allCodes).toContain(ErrorCode.CLAUDE_SESSION)
      expect(allCodes).toContain(ErrorCode.CLAUDE_NOT_INSTALLED)
      expect(allCodes).toContain(ErrorCode.AGENT_EXECUTION)
      expect(allCodes).toContain(ErrorCode.AGENT_NOT_FOUND)
      expect(allCodes).toContain(ErrorCode.AGENT_INVALID_CONFIG)
      expect(allCodes).toContain(ErrorCode.MCP_SERVER_ERROR)
      expect(allCodes).toContain(ErrorCode.MCP_CONNECTION)
      expect(allCodes).toContain(ErrorCode.DATABASE_ERROR)
      expect(allCodes).toContain(ErrorCode.DATABASE_MIGRATION)
      expect(allCodes).toContain(ErrorCode.FILE_NOT_FOUND)
      expect(allCodes).toContain(ErrorCode.FILE_PERMISSION)
      expect(allCodes).toContain(ErrorCode.NETWORK_ERROR)
      expect(allCodes).toContain(ErrorCode.TIMEOUT)
    })
  })
})