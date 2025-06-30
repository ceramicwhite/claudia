/**
 * Error handling utilities for UI components
 */

import { AppError, ErrorCode, ErrorHandler as BaseErrorHandler } from '@/lib/errors';
import { toast } from 'sonner';

export interface ErrorDisplayOptions {
  showToast?: boolean;
  fallbackMessage?: string;
  duration?: number;
  action?: {
    label: string;
    onClick: () => void;
  };
}

/**
 * Enhanced error handler for UI components
 */
export class UIErrorHandler extends BaseErrorHandler {
  /**
   * Display error to user with appropriate formatting
   */
  static displayError(
    error: unknown,
    options: ErrorDisplayOptions = {}
  ): AppError {
    const appError = this.handle(error);
    const {
      showToast = true,
      fallbackMessage,
      duration = 5000,
      action,
    } = options;
    
    if (showToast) {
      const message = this.getUserMessage(appError) || fallbackMessage || 'An error occurred';
      
      toast.error(message, {
        duration,
        action: action ? {
          label: action.label,
          onClick: action.onClick,
        } : undefined,
        description: this.getErrorDescription(appError),
      });
    }
    
    return appError;
  }
  
  /**
   * Get error description for display
   */
  private static getErrorDescription(error: AppError): string | undefined {
    // Don't show technical details for user-facing errors
    const userFacingCodes = [
      ErrorCode.UNAUTHORIZED,
      ErrorCode.FORBIDDEN,
      ErrorCode.NOT_FOUND,
    ];
    
    if (userFacingCodes.includes(error.code)) {
      return undefined;
    }
    
    // Show error code for technical errors
    return `Error code: ${error.code}`;
  }
  
  /**
   * Handle async operations with error handling
   */
  static async handleAsync<T>(
    operation: () => Promise<T>,
    options: ErrorDisplayOptions = {}
  ): Promise<T | null> {
    try {
      return await operation();
    } catch (error) {
      this.displayError(error, options);
      return null;
    }
  }
  
  /**
   * Create error boundary handler
   */
  static createBoundaryHandler(
    componentName: string
  ): (error: Error, errorInfo: React.ErrorInfo) => void {
    return (error: Error, errorInfo: React.ErrorInfo) => {
      console.error(`Error in ${componentName}:`, error, errorInfo);
      
      this.displayError(error, {
        fallbackMessage: `An error occurred in ${componentName}. Please refresh the page.`,
        action: {
          label: 'Refresh',
          onClick: () => window.location.reload(),
        },
      });
    };
  }
  
  /**
   * Handle form validation errors
   */
  static handleValidationError(
    error: unknown,
    fieldErrors?: Record<string, string>
  ): Record<string, string> {
    const appError = this.handle(error);
    
    if (appError.code === ErrorCode.VALIDATION && appError.details) {
      const details = appError.details as any;
      
      if (details.errors && Array.isArray(details.errors)) {
        const errors: Record<string, string> = {};
        
        for (const zodError of details.errors) {
          const field = zodError.path.join('.');
          errors[field] = zodError.message;
        }
        
        return { ...fieldErrors, ...errors };
      }
    }
    
    return {
      ...fieldErrors,
      _form: this.getUserMessage(appError),
    };
  }
}

/**
 * React hook for error handling
 */
export function useErrorHandler(
  defaultOptions: ErrorDisplayOptions = {}
) {
  const handleError = (error: unknown, options?: ErrorDisplayOptions) => {
    return UIErrorHandler.displayError(error, { ...defaultOptions, ...options });
  };
  
  const handleAsync = async <T>(
    operation: () => Promise<T>,
    options?: ErrorDisplayOptions
  ): Promise<T | null> => {
    return UIErrorHandler.handleAsync(operation, { ...defaultOptions, ...options });
  };
  
  return {
    handleError,
    handleAsync,
  };
}