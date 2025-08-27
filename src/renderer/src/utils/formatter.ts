/**
 * Date and time formatting utilities
 */

/**
 * Format a date to a human-readable relative time string
 */

/**
 * Get the largest time unit and its value from the difference in milliseconds
 * @param {number} diff - Difference in milliseconds
 * @returns {{value: number, unit: string} | null}
 */
function getRelativeTimeUnit(diff: number): { value: number; unit: string } | null {
  const units = [
    { name: 'year', ms: 365 * 24 * 60 * 60 * 1000 },
    { name: 'month', ms: 30 * 24 * 60 * 60 * 1000 },
    { name: 'week', ms: 7 * 24 * 60 * 60 * 1000 },
    { name: 'day', ms: 24 * 60 * 60 * 1000 },
    { name: 'hour', ms: 60 * 60 * 1000 },
    { name: 'minute', ms: 60 * 1000 },
    { name: 'second', ms: 1000 }
  ]

  for (const unit of units) {
    const value = Math.floor(diff / unit.ms)
    if (value > 0) {
      return { value, unit: unit.name }
    }
  }
  return null
}

/**
 * Format a date to a human-readable relative time string
 * @param {Date} date
 * @returns {string}
 */
export function formatRelativeTime(date: Date | number): string {
  const now = new Date()
  const diff = now.getTime() - (typeof date === 'number' ? date : date.getTime())

  if (diff < 31 * 1000) {
    return 'Just now'
  }

  const relative = getRelativeTimeUnit(diff)
  if (!relative) {
    return 'Just now'
  }

  const { value, unit } = relative
  return `${value} ${unit}${value > 1 ? 's' : ''} ago`
}

/**
 * Format a date to a short string (e.g., "Dec 15, 2023")
 */
export function formatShortDate(date: Date): string {
  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric'
  }).format(date)
}

/**
 * Format a date to a full string (e.g., "December 15, 2023 at 2:30 PM")
 */
export function formatFullDate(date: Date): string {
  return new Intl.DateTimeFormat('en-US', {
    month: 'long',
    day: 'numeric',
    year: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
    hour12: true
  }).format(date)
}

/**
 * Format a time duration in milliseconds to a human-readable string
 */
export function formatDuration(milliseconds: number): string {
  const seconds = Math.floor(milliseconds / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)

  if (days > 0) {
    const remainingHours = hours % 24
    return `${days}d ${remainingHours}h`
  } else if (hours > 0) {
    const remainingMinutes = minutes % 60
    return `${hours}h ${remainingMinutes}m`
  } else if (minutes > 0) {
    const remainingSeconds = seconds % 60
    return `${minutes}m ${remainingSeconds}s`
  } else {
    return `${seconds}s`
  }
}

/**
 * Format time only (e.g., "2:30 PM")
 */
export function formatTime(date: Date): string {
  return new Intl.DateTimeFormat('en-US', {
    hour: 'numeric',
    minute: '2-digit',
    hour12: true
  }).format(date)
}

/**
 * Format date and time in ISO-like format but user-friendly
 */
export function formatDateTime(date: Date): string {
  const today = new Date()
  const yesterday = new Date(today)
  yesterday.setDate(yesterday.getDate() - 1)

  const isToday = date.toDateString() === today.toDateString()
  const isYesterday = date.toDateString() === yesterday.toDateString()

  const timeStr = formatTime(date)

  if (isToday) {
    return `Today at ${timeStr}`
  } else if (isYesterday) {
    return `Yesterday at ${timeStr}`
  } else {
    return `${formatShortDate(date)} at ${timeStr}`
  }
}

/**
 * Check if a date is today
 */
export function isToday(date: Date): boolean {
  const today = new Date()
  return date.toDateString() === today.toDateString()
}

/**
 * Check if a date is within the last week
 */
export function isThisWeek(date: Date): boolean {
  const now = new Date()
  const weekAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000)
  return date >= weekAgo
}

/**
 * Check if a date is within the last month
 */
export function isThisMonth(date: Date): boolean {
  const now = new Date()
  const monthAgo = new Date(now.getTime() - 30 * 24 * 60 * 60 * 1000)
  return date >= monthAgo
}
