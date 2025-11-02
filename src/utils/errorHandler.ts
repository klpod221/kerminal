/**
 * Unified error handling utilities with retry logic
 */

export interface RetryOptions {
  /** Maximum number of retry attempts */
  maxRetries?: number;
  /** Delay between retries in milliseconds */
  retryDelay?: number;
  /** Exponential backoff multiplier */
  backoffMultiplier?: number;
  /** Maximum delay between retries in milliseconds */
  maxRetryDelay?: number;
  /** Function to determine if an error is retryable */
  isRetryable?: (error: unknown) => boolean;
}

export interface ErrorContext {
  /** Operation being performed */
  operation: string;
  /** Additional context information */
  context?: Record<string, unknown>;
}

/**
 * Default retry options
 */
const DEFAULT_RETRY_OPTIONS: Required<RetryOptions> = {
  maxRetries: 3,
  retryDelay: 1000,
  backoffMultiplier: 2,
  maxRetryDelay: 10000,
  isRetryable: (error: unknown) => {
    // Retry on network errors, timeouts, and 5xx server errors
    if (error && typeof error === "object") {
      if ("message" in error) {
        const message = String(error.message).toLowerCase();
        return (
          message.includes("network") ||
          message.includes("timeout") ||
          message.includes("connection") ||
          message.includes("econnrefused") ||
          message.includes("econnreset")
        );
      }
      if ("status" in error) {
        const status = Number(error.status);
        return status >= 500 && status < 600;
      }
    }
    return false;
  },
};

/**
 * Calculate delay for retry with exponential backoff
 * @param attempt - Current retry attempt (0-indexed)
 * @param options - Retry options
 * @returns Delay in milliseconds
 */
function calculateRetryDelay(
  attempt: number,
  options: Required<RetryOptions>,
): number {
  const delay =
    options.retryDelay * Math.pow(options.backoffMultiplier, attempt);
  return Math.min(delay, options.maxRetryDelay);
}

/**
 * Sleep for specified milliseconds
 * @param ms - Milliseconds to sleep
 */
function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Execute a function with retry logic
 * @param fn - Function to execute
 * @param options - Retry options
 * @param context - Error context for better error messages
 * @returns Result of the function execution
 * @throws Error if all retries fail
 */
export async function withRetry<T>(
  fn: () => Promise<T>,
  options: RetryOptions = {},
  context?: ErrorContext,
): Promise<T> {
  const retryOptions: Required<RetryOptions> = {
    ...DEFAULT_RETRY_OPTIONS,
    ...options,
    isRetryable: options.isRetryable || DEFAULT_RETRY_OPTIONS.isRetryable,
  };

  let lastError: unknown;
  const maxAttempts = retryOptions.maxRetries + 1;

  for (let attempt = 0; attempt < maxAttempts; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error;

      // Don't retry if it's the last attempt or error is not retryable
      if (attempt === maxAttempts - 1 || !retryOptions.isRetryable(error)) {
        break;
      }

      // Calculate delay and wait before retry
      const delay = calculateRetryDelay(attempt, retryOptions);
      await sleep(delay);
    }
  }

  // All retries failed, throw enhanced error
  throw enhanceError(lastError, context);
}

/**
 * Enhanced error with context information
 */
export class EnhancedError extends Error {
  constructor(
    message: string,
    public readonly originalError: unknown,
    public readonly context?: ErrorContext,
  ) {
    super(message);
    this.name = "EnhancedError";
    // Maintain stack trace
    if (originalError instanceof Error) {
      this.stack = originalError.stack;
    }
  }
}

/**
 * Enhance error with context information
 * @param error - Original error
 * @param context - Error context
 * @returns Enhanced error
 */
export function enhanceError(
  error: unknown,
  context?: ErrorContext,
): EnhancedError {
  const errorMessage = extractErrorMessage(error);
  const message = context
    ? `${context.operation} failed: ${errorMessage}`
    : errorMessage;

  return new EnhancedError(message, error, context);
}

/**
 * Extract error message from unknown error type
 * @param error - Error to extract message from
 * @returns Error message string
 */
export function extractErrorMessage(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }

  if (error && typeof error === "object") {
    if ("message" in error && typeof error.message === "string") {
      return error.message;
    }

    if (error instanceof Error) {
      return error.message;
    }

    if ("error" in error && typeof error.error === "string") {
      return error.error;
    }

    try {
      return JSON.stringify(error);
    } catch {}
  }

  return "An unknown error occurred";
}

/**
 * Get error type/category for better error handling
 * @param error - Error to categorize
 * @returns Error category
 */
export type ErrorCategory =
  | "network"
  | "authentication"
  | "authorization"
  | "validation"
  | "notFound"
  | "timeout"
  | "server"
  | "client"
  | "unknown";

/**
 * Categorize error for better handling
 * @param error - Error to categorize
 * @returns Error category
 */
export function categorizeError(error: unknown): ErrorCategory {
  if (!error || typeof error !== "object") {
    return "unknown";
  }

  const errorMessage =
    "message" in error && typeof error.message === "string"
      ? error.message.toLowerCase()
      : "";

  if (errorMessage.includes("network") || errorMessage.includes("connection")) {
    return "network";
  }
  if (errorMessage.includes("auth")) {
    return "authentication";
  }
  if (
    errorMessage.includes("permission") ||
    errorMessage.includes("forbidden")
  ) {
    return "authorization";
  }
  if (errorMessage.includes("validation") || errorMessage.includes("invalid")) {
    return "validation";
  }
  if (errorMessage.includes("not found") || errorMessage.includes("404")) {
    return "notFound";
  }
  if (errorMessage.includes("timeout")) {
    return "timeout";
  }
  if ("status" in error) {
    const status = Number(error.status);
    if (status >= 500) {
      return "server";
    }
    if (status >= 400) {
      return "client";
    }
  }

  return "unknown";
}

/**
 * Standardized error handler that logs and formats errors
 * @param error - Error to handle
 * @param context - Error context
 * @param showUserMessage - Whether to show user-facing message (default: true)
 * @returns Formatted error message for display
 */
export function handleError(
  error: unknown,
  context?: ErrorContext,
  showUserMessage = true,
): string {
  const enhancedError = enhanceError(error, context);
  const category = categorizeError(error);

  console.error("Error occurred:", {
    message: enhancedError.message,
    category,
    context: enhancedError.context,
    originalError: enhancedError.originalError,
  });

  if (!showUserMessage) {
    return enhancedError.message;
  }

  switch (category) {
    case "network":
      return context
        ? `Network error during ${context.operation.toLowerCase()}. Please check your connection.`
        : "Network error. Please check your connection.";
    case "authentication":
      return "Authentication failed. Please check your credentials.";
    case "authorization":
      return "You don't have permission to perform this action.";
    case "validation":
      return enhancedError.message;
    case "notFound":
      return context
        ? `${context.operation} not found.`
        : "Resource not found.";
    case "timeout":
      return context
        ? `${context.operation} timed out. Please try again.`
        : "Request timed out. Please try again.";
    case "server":
      return "Server error occurred. Please try again later.";
    case "client":
      return enhancedError.message;
    default:
      return enhancedError.message;
  }
}
