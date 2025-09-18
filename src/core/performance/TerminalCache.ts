/**
 * Terminal Cache Service
 * Provides intelligent caching for frequently accessed terminal data
 */

import { api } from "../../services/api";

/**
 * Cache entry with timestamp for TTL management
 */
interface CacheEntry<T> {
  data: T;
  timestamp: number;
}

/**
 * Cache statistics for monitoring
 */
export interface CacheStats {
  hits: number;
  misses: number;
  hitRate: number;
  totalEntries: number;
  memoryUsage: number; // approximate bytes
}

/**
 * Intelligent cache for terminal-related API calls
 * Reduces redundant API invocations and improves performance
 */
export class TerminalCache {
  private static instance: TerminalCache | null = null;

  // Cache storage
  private terminalInfoCache = new Map<string, CacheEntry<any>>();
  private bufferStatsCache: CacheEntry<any> | null = null;
  private terminalListCache: CacheEntry<any[]> | null = null;

  // Cache configuration
  private readonly DEFAULT_TTL = 5000; // 5 seconds
  private readonly BUFFER_STATS_TTL = 3000; // 3 seconds for stats
  private readonly TERMINAL_LIST_TTL = 2000; // 2 seconds for list

  // Statistics tracking
  private stats = {
    hits: 0,
    misses: 0,
  };

  /**
   * Get singleton instance
   */
  static getInstance(): TerminalCache {
    if (!TerminalCache.instance) {
      TerminalCache.instance = new TerminalCache();
    }
    return TerminalCache.instance;
  }

  /**
   * Get terminal info with caching
   * @param terminalId - Terminal identifier
   * @returns Promise of terminal info
   */
  async getTerminalInfo(terminalId: string): Promise<any> {
    const cacheKey = terminalId;
    const cached = this.terminalInfoCache.get(cacheKey);

    if (cached && this.isEntryValid(cached, this.DEFAULT_TTL)) {
      this.stats.hits++;
      return cached.data;
    }

    this.stats.misses++;
    try {
      const data = await api.call("get_terminal_info", {
        terminalId: terminalId,
      });
      this.terminalInfoCache.set(cacheKey, {
        data,
        timestamp: Date.now(),
      });
      return data;
    } catch (error) {
      console.error(`Failed to get terminal info for ${terminalId}:`, error);
      throw error;
    }
  }

  /**
   * Get buffer statistics with caching
   * @returns Promise of buffer statistics
   */
  async getBufferStats(): Promise<any> {
    if (
      this.bufferStatsCache &&
      this.isEntryValid(this.bufferStatsCache, this.BUFFER_STATS_TTL)
    ) {
      this.stats.hits++;
      return this.bufferStatsCache.data;
    }

    this.stats.misses++;
    try {
      const data = await api.call("get_buffer_stats");
      this.bufferStatsCache = {
        data,
        timestamp: Date.now(),
      };
      return data;
    } catch (error) {
      console.error("Failed to get buffer stats:", error);
      throw error;
    }
  }

  /**
   * Get terminal list with caching
   * @returns Promise of terminal list
   */
  async getTerminalList(): Promise<any[]> {
    if (
      this.terminalListCache &&
      this.isEntryValid(this.terminalListCache, this.TERMINAL_LIST_TTL)
    ) {
      this.stats.hits++;
      return this.terminalListCache.data;
    }

    this.stats.misses++;
    try {
      const data = await api.call("list_terminals");
      this.terminalListCache = {
        data,
        timestamp: Date.now(),
      };
      return data;
    } catch (error) {
      console.error("Failed to get terminal list:", error);
      throw error;
    }
  }

  /**
   * Check if terminal has buffer with caching
   * Note: This is cached with shorter TTL as buffer state changes frequently
   * @param terminalId - Terminal identifier
   * @returns Promise of boolean
   */
  async hasTerminalBuffer(terminalId: string): Promise<boolean> {
    // Buffer existence is cached for a shorter time (1 second)
    const cacheKey = `buffer_has_${terminalId}`;
    const cached = this.terminalInfoCache.get(cacheKey);

    if (cached && this.isEntryValid(cached, 1000)) {
      this.stats.hits++;
      return cached.data;
    }

    this.stats.misses++;
    try {
      const data = await api.call("has_terminal_buffer", {
        terminalId: terminalId,
      });
      this.terminalInfoCache.set(cacheKey, {
        data,
        timestamp: Date.now(),
      });
      return data;
    } catch (error) {
      console.error(
        `Failed to check buffer for terminal ${terminalId}:`,
        error,
      );
      throw error;
    }
  }

  /**
   * Invalidate cache for a specific terminal
   * @param terminalId - Terminal identifier
   */
  invalidateTerminal(terminalId: string): void {
    this.terminalInfoCache.delete(terminalId);
    this.terminalInfoCache.delete(`buffer_has_${terminalId}`);

    // Also invalidate terminal list cache since it contains this terminal
    this.terminalListCache = null;
  }

  /**
   * Invalidate buffer-related caches
   */
  invalidateBufferStats(): void {
    this.bufferStatsCache = null;
  }

  /**
   * Invalidate terminal list cache
   */
  invalidateTerminalList(): void {
    this.terminalListCache = null;
  }

  /**
   * Invalidate all caches
   */
  invalidateAll(): void {
    this.terminalInfoCache.clear();
    this.bufferStatsCache = null;
    this.terminalListCache = null;
  }

  /**
   * Get cache statistics
   * @returns Cache performance statistics
   */
  getStats(): CacheStats {
    const totalRequests = this.stats.hits + this.stats.misses;
    const hitRate = totalRequests > 0 ? this.stats.hits / totalRequests : 0;

    // Estimate memory usage (rough calculation)
    const memoryUsage = this.estimateMemoryUsage();

    return {
      hits: this.stats.hits,
      misses: this.stats.misses,
      hitRate,
      totalEntries:
        this.terminalInfoCache.size +
        (this.bufferStatsCache ? 1 : 0) +
        (this.terminalListCache ? 1 : 0),
      memoryUsage,
    };
  }

  /**
   * Reset cache statistics
   */
  resetStats(): void {
    this.stats.hits = 0;
    this.stats.misses = 0;
  }

  /**
   * Cleanup expired entries
   */
  cleanup(): void {
    // Clean terminal info cache
    for (const [key, entry] of this.terminalInfoCache.entries()) {
      if (!this.isEntryValid(entry, this.DEFAULT_TTL)) {
        this.terminalInfoCache.delete(key);
      }
    }

    // Clean buffer stats cache
    if (
      this.bufferStatsCache &&
      !this.isEntryValid(this.bufferStatsCache, this.BUFFER_STATS_TTL)
    ) {
      this.bufferStatsCache = null;
    }

    // Clean terminal list cache
    if (
      this.terminalListCache &&
      !this.isEntryValid(this.terminalListCache, this.TERMINAL_LIST_TTL)
    ) {
      this.terminalListCache = null;
    }
  }

  /**
   * Check if cache entry is still valid
   * @param entry - Cache entry to check
   * @param ttl - Time-to-live in milliseconds
   * @returns Whether entry is valid
   */
  private isEntryValid<T>(entry: CacheEntry<T>, ttl: number): boolean {
    return Date.now() - entry.timestamp < ttl;
  }

  /**
   * Estimate memory usage of cache (rough calculation)
   * @returns Estimated bytes
   */
  private estimateMemoryUsage(): number {
    let size = 0;

    // Estimate terminal info cache
    for (const [key, entry] of this.terminalInfoCache.entries()) {
      size += key.length * 2; // UTF-16 characters
      size += JSON.stringify(entry.data).length * 2;
      size += 8; // timestamp
    }

    // Estimate buffer stats cache
    if (this.bufferStatsCache) {
      size += JSON.stringify(this.bufferStatsCache.data).length * 2;
      size += 8; // timestamp
    }

    // Estimate terminal list cache
    if (this.terminalListCache) {
      size += JSON.stringify(this.terminalListCache.data).length * 2;
      size += 8; // timestamp
    }

    return size;
  }
}

// Export singleton instance getter
export const terminalCache = TerminalCache.getInstance();
