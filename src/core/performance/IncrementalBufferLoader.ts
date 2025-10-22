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
      const hasBuffer = await this.checkHasBuffer(terminalId);
      if (!hasBuffer) {
        return false;
      }

      terminal.clear();

      let currentLine = 0;
      let totalLines = 0;
      let currentChunk = 0;
      let isFirstChunk = true;

      while (true) {
        if (signal?.aborted) {
          throw new Error("Buffer loading was cancelled");
        }

        const chunk = await this.loadChunk(terminalId, currentLine, chunkSize);

        if (isFirstChunk) {
          totalLines = chunk.totalLines;
          isFirstChunk = false;
        }

        if (chunk.data.length === 0 && !chunk.hasMore) {
          break;
        }

        if (chunk.data.length > 0) {
          const dataToWrite =
            currentLine > 0 && !chunk.data.startsWith("\n")
              ? "\n" + chunk.data
              : chunk.data;

          terminal.write(dataToWrite);
        }

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

        currentLine = chunk.endLine;

        if (!chunk.hasMore) {
          break;
        }

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
      message.error(
        `Failed to load chunk for terminal ${terminalId}: ${error}`,
      );
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
