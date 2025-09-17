/**
 * Get the largest time unit and its value from the difference in milliseconds
 * @param {number} diff - Difference in milliseconds
 * @returns {{value: number, unit: string} | null}
 */
function getRelativeTimeUnit(
  diff: number,
): { value: number; unit: string } | null {
  const units = [
    { name: "year", ms: 365 * 24 * 60 * 60 * 1000 },
    { name: "month", ms: 30 * 24 * 60 * 60 * 1000 },
    { name: "week", ms: 7 * 24 * 60 * 60 * 1000 },
    { name: "day", ms: 24 * 60 * 60 * 1000 },
    { name: "hour", ms: 60 * 60 * 1000 },
    { name: "minute", ms: 60 * 1000 },
    { name: "second", ms: 1000 },
  ];

  for (const unit of units) {
    const value = Math.floor(diff / unit.ms);
    if (value > 0) {
      return { value, unit: unit.name };
    }
  }
  return null;
}

/**
 * Format a date to a human-readable relative time string
 * @param {Date} date
 * @returns {string}
 */
export function formatRelativeTime(date: Date | number): string {
  const now = new Date();
  const diff =
    now.getTime() - (typeof date === "number" ? date : date.getTime());

  if (diff < 31 * 1000) {
    return "Just now";
  }

  const relative = getRelativeTimeUnit(diff);
  if (!relative) {
    return "Just now";
  }

  const { value, unit } = relative;
  return `${value} ${unit}${value > 1 ? "s" : ""} ago`;
}

/**
 * Format a date to a short string (e.g., "Dec 15, 2023")
 */
export function formatShortDate(date: Date): string {
  return new Intl.DateTimeFormat("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(date);
}

/**
 * Format bytes to human readable format
 * @param {number} bytes - Bytes to format
 * @param {number} decimals - Number of decimal places
 * @returns {string}
 */
export function formatBytes(bytes: number, decimals: number = 2): string {
  if (bytes === 0) return "0 Bytes";

  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ["Bytes", "KB", "MB", "GB", "TB", "PB"];

  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + " " + sizes[i];
}

/**
 * Format uptime in seconds to human readable format
 * @param {number} seconds - Uptime in seconds
 * @returns {string}
 */
export function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);

  const parts = [];
  if (days > 0) parts.push(`${days}d`);
  if (hours > 0) parts.push(`${hours}h`);
  if (minutes > 0) parts.push(`${minutes}m`);

  return parts.length > 0 ? parts.join(" ") : "< 1m";
}

/**
 * Format temperature with unit
 * @param {number} temperature - Temperature in Celsius
 * @returns {string}
 */
export function formatTemperature(temperature: number): string {
  return `${temperature.toFixed(1)}°C`;
}

/**
 * Format percentage
 * @param {number} value - Value to format as percentage
 * @param {number} decimals - Number of decimal places
 * @returns {string}
 */
export function formatPercentage(value: number, decimals: number = 1): string {
  return `${value.toFixed(decimals)}%`;
}

/**
 * Format CPU frequency
 * @param {number} frequency - Frequency in MHz
 * @returns {string}
 */
export function formatFrequency(frequency: number): string {
  if (frequency >= 1000) {
    return `${(frequency / 1000).toFixed(2)} GHz`;
  }
  return `${frequency} MHz`;
}
