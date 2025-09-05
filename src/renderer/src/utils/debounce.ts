/**
 * Debouncing utility for clipboard operations
 */

/**
 * Creates a debounced function that delays invoking func until after wait milliseconds
 * have elapsed since the last time the debounced function was invoked.
 */
export function debounce<T extends (...args: unknown[]) => unknown>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout> | null = null

  return (...args: Parameters<T>) => {
    if (timeoutId !== null) {
      clearTimeout(timeoutId)
    }

    timeoutId = setTimeout(() => func(...args), wait)
  }
}

/**
 * Creates a throttled function that only invokes func at most once per every wait milliseconds.
 */
export function throttle<T extends (...args: unknown[]) => unknown>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let lastTime = 0

  return (...args: Parameters<T>) => {
    const now = Date.now()
    if (now - lastTime >= wait) {
      lastTime = now
      func(...args)
    }
  }
}

/**
 * Clipboard operation manager with rate limiting
 */
export class ClipboardManager {
  private lastCopyTime = 0
  private lastPasteTime = 0
  private readonly copyThrottleMs: number
  private readonly pasteThrottleMs: number

  constructor(copyThrottleMs = 100, pasteThrottleMs = 200) {
    this.copyThrottleMs = copyThrottleMs
    this.pasteThrottleMs = pasteThrottleMs
  }

  /**
   * Throttled copy operation
   * @param text - Text to copy
   * @param copyFn - Function to perform the copy
   * @returns {boolean} - Whether copy was performed
   */
  copy(text: string, copyFn: (text: string) => void): boolean {
    const now = Date.now()
    if (now - this.lastCopyTime >= this.copyThrottleMs) {
      this.lastCopyTime = now
      copyFn(text)
      return true
    }
    return false
  }

  /**
   * Throttled paste operation
   * @param pasteFn - Function to perform the paste
   * @returns {boolean} - Whether paste was performed
   */
  paste(pasteFn: () => void): boolean {
    const now = Date.now()
    if (now - this.lastPasteTime >= this.pasteThrottleMs) {
      this.lastPasteTime = now
      pasteFn()
      return true
    }
    return false
  }

  /**
   * Reset throttle timers
   */
  reset(): void {
    this.lastCopyTime = 0
    this.lastPasteTime = 0
  }
}
