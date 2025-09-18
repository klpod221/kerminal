/**
 * Terminal Buffer Manager for Tauri Application
 * Manages terminal output buffers and synchronizes with Rust backend
 */
import * as bufferService from "../services/buffer";
import type { BufferStats } from "../services/buffer";

export class TerminalBufferManager {
  private static instance: TerminalBufferManager;
  private readonly localBuffers: Map<string, string[]> = new Map();
  private readonly MAX_LOCAL_BUFFER_LINES = 500;

  /**
   * Get singleton instance
   */
  static getInstance(): TerminalBufferManager {
    if (!TerminalBufferManager.instance) {
      TerminalBufferManager.instance = new TerminalBufferManager();
    }
    return TerminalBufferManager.instance;
  }

  /**
   * Save data to local buffer
   * @param terminalId - Terminal identifier
   * @param data - Terminal output data
   */
  saveToLocalBuffer(terminalId: string, data: string): void {
    try {
      if (!data || typeof data !== "string") {
        return;
      }

      // Initialize buffer if not exists
      if (!this.localBuffers.has(terminalId)) {
        this.localBuffers.set(terminalId, []);
      }

      const buffer = this.localBuffers.get(terminalId)!;

      // Split data by lines and add to buffer
      const lines = data.split("\n");

      // If the data doesn't end with newline, the last element should be merged with existing last line
      if (buffer.length > 0 && !data.startsWith("\n") && lines.length > 0) {
        buffer[buffer.length - 1] += lines[0];
        lines.shift(); // Remove first element as it's already merged
      }

      // Add remaining lines
      buffer.push(...lines);

      // Trim buffer to prevent memory overflow
      this.trimLocalBuffer(buffer);
    } catch (error) {
      console.error(
        `Failed to save to local buffer for terminal ${terminalId}:`,
        error,
      );
    }
  }

  /**
   * Get buffer as string from Rust backend
   * @param terminalId - Terminal identifier
   * @returns Promise of buffer string
   */
  async getBufferFromBackend(terminalId: string): Promise<string> {
    try {
      return await bufferService.getTerminalBuffer(terminalId);
    } catch (error) {
      console.error(
        `Failed to get buffer string from backend for terminal ${terminalId}:`,
        error,
      );
      return "";
    }
  }

  /**
   * Check if terminal has buffer in Rust backend
   * @param terminalId - Terminal identifier
   * @returns Promise of boolean
   */
  async hasBufferInBackend(terminalId: string): Promise<boolean> {
    try {
      return await bufferService.hasTerminalBuffer(terminalId);
    } catch (error) {
      console.error(
        `Failed to check buffer in backend for terminal ${terminalId}:`,
        error,
      );
      return false;
    }
  }

  /**
   * Get buffer statistics from Rust backend
   * @returns Promise of buffer statistics
   */
  async getBufferStats(): Promise<BufferStats> {
    try {
      return await bufferService.getBufferStats();
    } catch (error) {
      console.error("Failed to get buffer stats from backend:", error);
      return { totalTerminals: 0, totalLines: 0, memoryUsage: 0 };
    }
  }

  /**
   * Clear local buffer for specific terminal
   * @param terminalId - Terminal identifier
   */
  clearLocalBuffer(terminalId: string): void {
    this.localBuffers.delete(terminalId);
  }

  /**
   * Restore terminal buffer from Rust backend
   * @param terminalId - Terminal identifier
   * @param terminal - xterm.js Terminal instance
   * @returns Promise of success boolean
   */
  async restoreBuffer(
    terminalId: string,
    terminal: { clear: () => void; write: (data: string) => void },
  ): Promise<boolean> {
    try {
      // Check if buffer exists in backend
      const hasBuffer = await this.hasBufferInBackend(terminalId);
      if (!hasBuffer) {
        return false;
      }

      // Get buffer from backend
      const bufferString = await this.getBufferFromBackend(terminalId);
      if (!bufferString) {
        return false;
      }

      // Clear terminal and write buffer
      terminal.clear();
      terminal.write(bufferString);

      return true;
    } catch (error) {
      console.error(
        `Failed to restore buffer for terminal ${terminalId}:`,
        error,
      );
      return false;
    }
  }

  /**
   * Trigger cleanup of orphaned buffers in Rust backend
   */
  async triggerCleanup(): Promise<void> {
    try {
      // Get list of active terminal IDs from terminal manager first
      const activeTerminals = await bufferService.listTerminals();
      const activeTerminalIds = Array.isArray(activeTerminals)
        ? activeTerminals.map((t: any) => t.id)
        : [];

      await bufferService.cleanupTerminalBuffers(activeTerminalIds);
    } catch (error) {
      console.error("Failed to trigger buffer cleanup:", error);
    }
  }

  /**
   * Get combined buffer stats (local + backend)
   * @returns Promise of combined statistics
   */
  async getCombinedStats(): Promise<{
    local: { terminals: number; lines: number };
    main: BufferStats;
  }> {
    try {
      const mainStats = await this.getBufferStats();

      let localLines = 0;
      for (const buffer of this.localBuffers.values()) {
        localLines += buffer.length;
      }

      return {
        local: {
          terminals: this.localBuffers.size,
          lines: localLines,
        },
        main: mainStats,
      };
    } catch (error) {
      console.error("Failed to get combined stats:", error);
      return {
        local: { terminals: 0, lines: 0 },
        main: { totalTerminals: 0, totalLines: 0, memoryUsage: 0 },
      };
    }
  }

  /**
   * Cleanup all local buffers
   */
  cleanup(): void {
    this.localBuffers.clear();
  }

  /**
   * Trim local buffer to prevent memory overflow
   * @param buffer - Buffer array to trim
   */
  private trimLocalBuffer(buffer: string[]): void {
    if (buffer.length > this.MAX_LOCAL_BUFFER_LINES) {
      const trimCount = buffer.length - this.MAX_LOCAL_BUFFER_LINES;
      buffer.splice(0, trimCount);
    }
  }
}
