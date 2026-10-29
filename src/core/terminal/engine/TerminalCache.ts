// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * Terminal Cache Service
 * Provides intelligent caching for frequently accessed terminal data
 */

import { api } from "../../../services/api";
import type { TerminalInfo } from "../../../types/panel";

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

  private readonly terminalInfoCache = new Map<string, CacheEntry<TerminalInfo>>();
  private terminalListCache: CacheEntry<TerminalInfo[]> | null = null;

  private readonly DEFAULT_TTL = 5000; // 5 seconds
  private readonly TERMINAL_LIST_TTL = 2000; // 2 seconds for list

  private readonly stats = {
    hits: 0,
    misses: 0,
  };

  /**
   * Get singleton instance
   */
  static getInstance(): TerminalCache {
    TerminalCache.instance ??= new TerminalCache();
    return TerminalCache.instance;
  }

  /**
   * Get terminal info with caching
   * @param terminalId - Terminal identifier
   * @returns Promise of terminal info
   */
  async getTerminalInfo(terminalId: string): Promise<TerminalInfo> {
    const cacheKey = terminalId;
    const cached = this.terminalInfoCache.get(cacheKey);

    if (cached && this.isEntryValid(cached, this.DEFAULT_TTL)) {
      this.stats.hits++;
      return cached.data;
    }

    this.stats.misses++;
    try {
      const data = await api.call<TerminalInfo>("get_terminal_info", {
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
   * Get terminal list with caching
   * @returns Promise of terminal list
   */
  async getTerminalList(): Promise<TerminalInfo[]> {
    if (
      this.terminalListCache &&
      this.isEntryValid(this.terminalListCache, this.TERMINAL_LIST_TTL)
    ) {
      this.stats.hits++;
      return this.terminalListCache.data;
    }

    this.stats.misses++;
    try {
      const data = await api.call<TerminalInfo[]>("list_terminals");
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
   * Invalidate cache for a specific terminal
   * @param terminalId - Terminal identifier
   */
  invalidateTerminal(terminalId: string): void {
    this.terminalInfoCache.delete(terminalId);
    this.terminalListCache = null;
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
    this.terminalListCache = null;
  }

  /**
   * Get cache statistics
   * @returns Cache performance statistics
   */
  getStats(): CacheStats {
    const totalRequests = this.stats.hits + this.stats.misses;
    const hitRate = totalRequests > 0 ? this.stats.hits / totalRequests : 0;

    const memoryUsage = this.estimateMemoryUsage();

    return {
      hits: this.stats.hits,
      misses: this.stats.misses,
      hitRate,
      totalEntries:
        this.terminalInfoCache.size +
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
    for (const [key, entry] of this.terminalInfoCache.entries()) {
      if (!this.isEntryValid(entry, this.DEFAULT_TTL)) {
        this.terminalInfoCache.delete(key);
      }
    }

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

    for (const [key, entry] of this.terminalInfoCache.entries()) {
      size += key.length * 2; // UTF-16 characters
      size += JSON.stringify(entry.data).length * 2;
      size += 8; // timestamp
    }

    if (this.terminalListCache) {
      size += JSON.stringify(this.terminalListCache.data).length * 2;
      size += 8; // timestamp
    }

    return size;
  }
}

export const terminalCache = TerminalCache.getInstance();
