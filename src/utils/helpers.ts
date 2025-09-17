/**
 * Debounce function to limit the rate at which a function can fire.
 * @param func - The function to debounce.
 * @param wait - The number of milliseconds to wait before invoking the function.
 * @returns A debounced version of the input function.
 */
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number,
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout> | null = null;

  return (...args: Parameters<T>) => {
    if (timeoutId !== null) {
      clearTimeout(timeoutId);
    }

    timeoutId = setTimeout(() => func(...args), wait);
  };
}

/**
 * Extract error message from error object
 * @param error - The error object
 * @param fallbackMessage - Fallback message if no specific error message is found
 * @returns The error message to display
 */
export function getErrorMessage(error: unknown, fallbackMessage: string): string {
  if (typeof error === 'string') {
    return error;
  }
  
  if (error && typeof error === 'object') {
    // Handle Tauri error format
    if ('message' in error && typeof error.message === 'string') {
      return error.message;
    }
    
    // Handle standard Error objects
    if (error instanceof Error) {
      return error.message;
    }
    
    // Handle other object formats
    if ('error' in error && typeof error.error === 'string') {
      return error.error;
    }
  }
  
  return fallbackMessage;
}
