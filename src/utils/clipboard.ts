/**
 * Clipboard utilities for enhanced copy/paste functionality
 */

/**
 * Interface for clipboard validation options
 */
export interface ClipboardValidationOptions {
  maxLength?: number
  allowedPatterns?: RegExp[]
  blockedPatterns?: RegExp[]
  multilineConfirm?: boolean
}

/**
 * Validates clipboard content before pasting into terminal
 * @param text - Text to validate
 * @param options - Validation options
 * @returns {boolean} - Whether text is safe to paste
 */
export function validateClipboardContent(
  text: string,
  options: ClipboardValidationOptions = {}
): boolean {
  if (!text) return false

  const {
    maxLength = 100000, // Default max 100k characters
    allowedPatterns = [],
    blockedPatterns = [],
    multilineConfirm = false
  } = options

  // Check length
  if (text.length > maxLength) {
    return false
  }

  // Check for blocked patterns (dangerous commands)
  const dangerousPatterns = [
    /rm\s+-rf\s+\//, // rm -rf /
    /:\(\)\{.*\};:/, // Fork bomb pattern
    /chmod.*777/, // Dangerous chmod
    ...blockedPatterns
  ]

  for (const pattern of dangerousPatterns) {
    if (pattern.test(text)) {
      return false
    }
  }

  // Check allowed patterns if specified
  if (allowedPatterns.length > 0) {
    const isAllowed = allowedPatterns.some((pattern) => pattern.test(text))
    if (!isAllowed) {
      return false
    }
  }

  // Check for multiline content
  if (multilineConfirm && text.includes('\n')) {
    // Show a confirmation dialog
  }

  return true
}

/**
 * Sanitizes clipboard data for safe terminal insertion
 * @param text - Raw clipboard text
 * @returns {string} - Sanitized text
 */
export function sanitizeForTerminal(text: string): string {
  if (!text) return ''

  // Remove null bytes and other dangerous control characters
  let sanitized = text.replace(/\0/g, '')

  // Normalize line endings
  sanitized = sanitized.replace(/\r\n/g, '\n').replace(/\r/g, '\n')

  // Remove or escape potentially dangerous escape sequences
  // Keep basic ones but remove dangerous ones
  const ESC_CHAR = String.fromCharCode(0x1b)
  const ansiEscapeRegex = new RegExp(`${ESC_CHAR}\\[[0-9;]*[a-zA-Z]`, 'g')

  // Remove control characters (except \n=10 and \t=9)
  const controlCharsToRemove = [
    0,
    1,
    2,
    3,
    4,
    5,
    6,
    7,
    8, // \x00-\x08
    11,
    12, // \x0B, \x0C
    14,
    15,
    16,
    17,
    18,
    19,
    20,
    21,
    22,
    23,
    24,
    25,
    26,
    27,
    28,
    29,
    30,
    31, // \x0E-\x1F
    127 // \x7F
  ]

  sanitized = sanitized.replace(ansiEscapeRegex, '') // Remove ANSI escape sequences

  // Remove control characters
  for (const code of controlCharsToRemove) {
    const char = String.fromCharCode(code)
    sanitized = sanitized.split(char).join('')
  }

  return sanitized
}

/**
 * Formats text for better terminal display
 * @param text - Text to format
 * @returns {string} - Formatted text
 */
export function formatForTerminal(text: string): string {
  if (!text) return ''

  // Handle common formatting issues
  let formatted = text.trim()

  // Ensure single trailing newline for commands
  if (formatted && !formatted.endsWith('\n')) {
    // Only add newline if it looks like a command
    const commandPattern = /^[a-zA-Z].*[^\\]$/
    if (commandPattern.exec(formatted)) {
      formatted += '\n'
    }
  }

  return formatted
}

/**
 * Comprehensive clipboard processing pipeline
 * @param text - Raw clipboard text
 * @param options - Validation and processing options
 * @returns {string | null} - Processed text or null if invalid
 */
export function processClipboardText(
  text: string,
  options: ClipboardValidationOptions = {}
): string | null {
  if (!text) return null

  // Validate content
  if (!validateClipboardContent(text, options)) {
    return null
  }

  // Sanitize content
  const sanitized = sanitizeForTerminal(text)
  if (!sanitized) return null

  // Format for terminal
  const formatted = formatForTerminal(sanitized)

  return formatted
}
