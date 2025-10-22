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
export function getErrorMessage(
  error: unknown,
  fallbackMessage: string,
): string {
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
  }

  return fallbackMessage;
}

/**
 * Convert byte array to string
 */
export const bytesToString = (bytes: number[]): string => {
  return new TextDecoder().decode(new Uint8Array(bytes));
};

/**
 * Safe JSON parse with fallback value
 * @param json - JSON string to parse
 * @param fallback - Fallback value if parsing fails
 * @returns Parsed value or fallback
 */
export function safeJsonParse<T>(
  json: string | null | undefined,
  fallback: T,
): T {
  if (!json) return fallback;
  try {
    return JSON.parse(json) as T;
  } catch {
    return fallback;
  }
}

/**
 * Safe JSON stringify with fallback
 * @param value - Value to stringify
 * @param fallback - Fallback string if stringify fails
 * @returns Stringified value or fallback
 */
export function safeJsonStringify(value: unknown, fallback = ""): string {
  try {
    return JSON.stringify(value);
  } catch {
    return fallback;
  }
}

/**
 * Truncate text with ellipsis
 * @param text - Text to truncate
 * @param maxLength - Maximum length before truncation
 * @param ellipsis - Ellipsis string to append
 * @returns Truncated text
 */
export function truncateText(
  text: string,
  maxLength: number,
  ellipsis = "...",
): string {
  if (!text || text.length <= maxLength) return text;
  return text.substring(0, maxLength - ellipsis.length) + ellipsis;
}

/**
 * Check if a timestamp is recently active (within specified minutes)
 * @param timestamp - Timestamp to check (Date, number, or string)
 * @param withinMinutes - Number of minutes to consider as recent (default: 5)
 * @returns True if timestamp is within the specified minutes
 */
export function isRecentlyActive(
  timestamp: Date | number | string,
  withinMinutes = 5,
): boolean {
  const time =
    typeof timestamp === "number" ? timestamp : new Date(timestamp).getTime();
  const now = Date.now();
  return now - time < withinMinutes * 60 * 1000;
}

/**
 * Case-insensitive string includes check
 * @param text - Text to search in
 * @param query - Query to search for
 * @returns True if text includes query (case-insensitive)
 */
export function caseInsensitiveIncludes(
  text: string | null | undefined,
  query: string,
): boolean {
  if (!text) return false;
  return text.toLowerCase().includes(query.toLowerCase());
}

/**
 * Get current timestamp in ISO format
 * @returns ISO timestamp string
 */
export function getCurrentTimestamp(): string {
  return new Date().toISOString();
}
