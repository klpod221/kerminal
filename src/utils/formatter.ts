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
