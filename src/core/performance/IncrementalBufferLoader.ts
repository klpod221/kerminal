import { api } from "../../services/api";
import { message } from "../../utils/message";

/**
 * Simple terminal interface for buffer loading
 */
export interface SimpleTerminal {
  clear: () => void;
  write: (data: string) => void;
}

/**
 * Buffer chunk interface matching Rust backend
 */
export interface TerminalBufferChunk {
  terminalId: string;
  startLine: number;
  endLine: number;
  totalLines: number;
  data: string;
  hasMore: boolean;
}

/**
 * Progress callback for buffer loading
 */
export type LoadProgressCallback = (progress: {
  loadedLines: number;
  totalLines: number;
  percentage: number;
  currentChunk: number;
  totalChunks: number;
}) => void;

/**
 * Options for incremental buffer loading
 */
export interface LoadOptions {
  chunkSize?: number;
  delayBetweenChunks?: number;
  onProgress?: LoadProgressCallback;
  signal?: AbortSignal; // For cancellation
}

/**
 * Incremental buffer loader for progressive terminal buffer restoration
 */
export class IncrementalBufferLoader {
  private readonly DEFAULT_CHUNK_SIZE = 100; // lines per chunk
  private readonly DEFAULT_DELAY = 5; // ms between chunks

  /**
   * Load buffer incrementally with progress feedback
   * @param terminalId - Terminal identifier
   * @param terminal - Simple terminal instance with clear() and write() methods
   * @param options - Load options
   * @returns Promise resolving to success status
   */
  async loadBuffer(
    terminalId: string,
    terminal: SimpleTerminal,
    options: LoadOptions = {},
  ): Promise<boolean> {
    const {
      chunkSize = this.DEFAULT_CHUNK_SIZE,
      delayBetweenChunks = this.DEFAULT_DELAY,
      onProgress,
      signal,
    } = options;

    try {
      // Check if terminal has buffer first
      const hasBuffer = await this.checkHasBuffer(terminalId);
      if (!hasBuffer) {
        return false;
      }

      // Clear terminal before loading
      terminal.clear();

      let currentLine = 0;
      let totalLines = 0;
      let currentChunk = 0;
      let isFirstChunk = true;

      while (true) {
        // Check for cancellation
        if (signal?.aborted) {
          throw new Error("Buffer loading was cancelled");
        }

        // Load chunk
        const chunk = await this.loadChunk(terminalId, currentLine, chunkSize);

        if (isFirstChunk) {
          totalLines = chunk.totalLines;
          isFirstChunk = false;
        }

        // If no data in this chunk, we're done
        if (chunk.data.length === 0 && !chunk.hasMore) {
          break;
        }

        // Write chunk data to terminal
        if (chunk.data.length > 0) {
          // Add newline between chunks if not the first chunk and data doesn't start with newline
          const dataToWrite =
            currentLine > 0 && !chunk.data.startsWith("\n")
              ? "\n" + chunk.data
              : chunk.data;

          terminal.write(dataToWrite);
        }

        // Update progress
        currentChunk++;
        const loadedLines = chunk.endLine;
        const estimatedTotalChunks = Math.ceil(totalLines / chunkSize);

        if (onProgress) {
          onProgress({
            loadedLines,
            totalLines,
            percentage: totalLines > 0 ? (loadedLines / totalLines) * 100 : 0,
            currentChunk,
            totalChunks: estimatedTotalChunks,
          });
        }

        // Move to next chunk
        currentLine = chunk.endLine;

        // Check if we have more data
        if (!chunk.hasMore) {
          break;
        }

        // Small delay to prevent blocking UI
        if (delayBetweenChunks > 0) {
          await this.delay(delayBetweenChunks);
        }
      }

      return true;
    } catch (error) {
      message.error(
        `Failed to load buffer incrementally for terminal ${terminalId}: ${error}`,
      );
      return false;
    }
  }

  /**
   * Get buffer size information without loading
   * @param terminalId - Terminal identifier
   * @returns Buffer size info or null
   */
  async getBufferInfo(
    terminalId: string,
  ): Promise<{ totalLines: number } | null> {
    try {
      const hasBuffer = await this.checkHasBuffer(terminalId);
      if (!hasBuffer) {
        return null;
      }

      // Load just the first chunk to get total lines info
      const chunk = await this.loadChunk(terminalId, 0, 1);
      return { totalLines: chunk.totalLines };
    } catch (error) {
      message.error(
        `Failed to get buffer info for terminal ${terminalId}: ${error}`,
      );
      return null;
    }
  }

  /**
   * Load a single chunk of buffer data
   * @param terminalId - Terminal identifier
   * @param startLine - Starting line number
   * @param chunkSize - Number of lines to load
   * @returns Buffer chunk
   */
  private async loadChunk(
    terminalId: string,
    startLine: number,
    chunkSize: number,
  ): Promise<TerminalBufferChunk> {
    try {
      const chunk = await api.call<TerminalBufferChunk>(
        "get_terminal_buffer_chunk",
        {
          terminalId: terminalId,
          startLine: startLine,
          chunkSize: chunkSize,
        },
      );

      return {
        terminalId: chunk.terminalId || terminalId,
        startLine: chunk.startLine || startLine,
        endLine: chunk.endLine || startLine,
        totalLines: chunk.totalLines || 0,
        data: chunk.data || "",
        hasMore: chunk.hasMore || false,
      };
    } catch (error) {
      message.error(`Failed to load chunk for terminal ${terminalId}: ${error}`);
      // Return empty chunk on error
      return {
        terminalId,
        startLine,
        endLine: startLine,
        totalLines: 0,
        data: "",
        hasMore: false,
      };
    }
  }

  /**
   * Check if terminal has buffer
   * @param terminalId - Terminal identifier
   * @returns Whether terminal has buffer
   */
  private async checkHasBuffer(terminalId: string): Promise<boolean> {
    try {
      return await api.call<boolean>("has_terminal_buffer", {
        terminalId: terminalId,
      });
    } catch (error) {
      message.error(
        `Failed to check buffer for terminal ${terminalId}: ${error}`,
      );
      return false;
    }
  }

  /**
   * Delay execution for specified milliseconds
   */
  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}
