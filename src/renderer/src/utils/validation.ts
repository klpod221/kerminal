/**
 * Common validation utilities
 */

/**
 * Check if a string is a valid email address
 */
export function isValidEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
  return emailRegex.test(email.trim())
}

/**
 * Check if a string is a valid URL
 */
export function isValidUrl(url: string): boolean {
  try {
    new URL(url)
    return true
  } catch {
    return false
  }
}

/**
 * Check if a value is a valid port number
 */
export function isValidPort(port: number | string): boolean {
  const portNum = typeof port === 'string' ? parseInt(port, 10) : port
  return !isNaN(portNum) && portNum >= 1 && portNum <= 65535
}

/**
 * Check if a string is a valid hostname
 */
export function isValidHostname(hostname: string): boolean {
  if (!hostname || hostname.length > 253) return false

  // Check for valid characters and format
  const hostnameRegex =
    /^[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$/
  return hostnameRegex.test(hostname)
}

/**
 * Check if a string is a valid IP address (IPv4 or IPv6)
 */
export function isValidIP(ip: string): boolean {
  return isValidIPv4(ip) || isValidIPv6(ip)
}

/**
 * Check if a string is a valid IPv4 address
 */
export function isValidIPv4(ip: string): boolean {
  const ipv4Regex = /^(\d{1,3}\.){3}\d{1,3}$/
  if (!ipv4Regex.test(ip)) return false

  const parts = ip.split('.')
  return parts.every((part) => {
    const num = parseInt(part, 10)
    return num >= 0 && num <= 255
  })
}

/**
 * Check if a string is a valid IPv6 address
 */
export function isValidIPv6(ip: string): boolean {
  const ipv6Regex = /^([0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}$/
  const ipv6CompressedRegex = /^([0-9a-fA-F]{1,4}:)+::([0-9a-fA-F]{1,4}:)*[0-9a-fA-F]{1,4}$/

  return ipv6Regex.test(ip) || ipv6CompressedRegex.test(ip)
}

/**
 * Validate required field
 */
export function isRequired(value: unknown): boolean {
  if (value === null || value === undefined) return false
  if (typeof value === 'string') return value.trim().length > 0
  if (Array.isArray(value)) return value.length > 0
  return true
}

/**
 * Validate minimum length
 */
export function hasMinLength(value: string, minLength: number): boolean {
  return value.length >= minLength
}

/**
 * Validate maximum length
 */
export function hasMaxLength(value: string, maxLength: number): boolean {
  return value.length <= maxLength
}

/**
 * Validate that value is within a numeric range
 */
export function isInRange(value: number, min: number, max: number): boolean {
  return value >= min && value <= max
}

/**
 * Validate that a string matches a pattern
 */
export function matchesPattern(value: string, pattern: RegExp): boolean {
  return pattern.test(value)
}

/**
 * Validate file path (basic check)
 */
export function isValidFilePath(path: string): boolean {
  if (!path || path.trim().length === 0) return false

  // Check for invalid characters
  const invalidChars = /[<>:"|?*]/
  if (invalidChars.test(path)) return false

  // Check for relative path indicators
  if (path.includes('..')) return false

  return true
}

/**
 * Validate SSH key path
 */
export function isValidSSHKeyPath(path: string): boolean {
  if (!isValidFilePath(path)) return false

  // Common SSH key file patterns
  const sshKeyPatterns = [/id_rsa$/, /id_dsa$/, /id_ecdsa$/, /id_ed25519$/, /\.pem$/, /\.key$/]

  return sshKeyPatterns.some((pattern) => pattern.test(path.toLowerCase()))
}

/**
 * Create a validation result object
 */
export interface ValidationResult {
  valid: boolean
  errors: string[]
  warnings?: string[]
}

/**
 * Create a validator function that combines multiple validation rules
 */
export function createValidator<T>(
  rules: Array<{
    validate: (value: T) => boolean
    message: string
    type?: 'error' | 'warning'
  }>
): (value: T) => ValidationResult {
  return (value: T): ValidationResult => {
    const errors: string[] = []
    const warnings: string[] = []

    for (const rule of rules) {
      if (!rule.validate(value)) {
        if (rule.type === 'warning') {
          warnings.push(rule.message)
        } else {
          errors.push(rule.message)
        }
      }
    }

    return {
      valid: errors.length === 0,
      errors,
      warnings
    }
  }
}
