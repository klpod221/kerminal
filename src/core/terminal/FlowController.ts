/**
 * Flow Controller for Terminal Output
 *
 * Implements watermark-based flow control and batched rendering
 * to prevent xterm.js WriteBuffer overflow on large output.
 */

import type { Terminal } from "@xterm/xterm";

/** Flow control configuration */
interface FlowControllerConfig {
  highWatermark?: number;
  lowWatermark?: number;
  maxBatchSize?: number;
}

/** Flow control statistics */
export interface FlowStats {
  bufferedBytes: number;
  pendingChunks: number;
  isPaused: boolean;
  totalBytesWritten: number;
  writeCount: number;
}

/**
 * Manages terminal output flow to prevent buffer overflow.
 *
 * Uses watermark-based flow control:
 * - Pauses data reception when buffered data exceeds HIGH_WATERMARK
 * - Resumes when buffered data drops below LOW_WATERMARK
 * - Batches writes using requestAnimationFrame for optimal rendering
 */
export class FlowController {
  private readonly HIGH_WATERMARK: number;
  private readonly LOW_WATERMARK: number;
  private readonly MAX_BATCH_SIZE: number;

  private terminal: Terminal | null = null;
  private bufferedBytes = 0;
  private pendingChunks: Uint8Array[] = [];
  private renderScheduled = false;
  private isPaused = false;

  private totalBytesWritten = 0;
  private writeCount = 0;

  private onPauseCallback?: () => void;
  private onResumeCallback?: () => void;

  constructor(config: FlowControllerConfig = {}) {
    this.HIGH_WATERMARK = config.highWatermark ?? 256 * 1024; // 256KB
    this.LOW_WATERMARK = config.lowWatermark ?? 64 * 1024;    // 64KB
    this.MAX_BATCH_SIZE = config.maxBatchSize ?? 1024 * 1024; // 1MB per frame
  }

  /**
   * Attach terminal instance
   */
  attach(terminal: Terminal): void {
    this.terminal = terminal;
  }

  /**
   * Detach terminal and flush pending data
   */
  detach(): void {
    this.flush();
    this.terminal = null;
  }

  /**
   * Set callback for when flow control pauses
   */
  onPause(callback: () => void): void {
    this.onPauseCallback = callback;
  }

  /**
   * Set callback for when flow control resumes
   */
  onResume(callback: () => void): void {
    this.onResumeCallback = callback;
  }

  /**
   * Queue data for writing to terminal
   * @param data - Data to write (string or Uint8Array)
   * @returns Whether the caller should continue sending data
   */
  write(data: string | Uint8Array): boolean {
    if (!this.terminal) return true;

    const chunk = typeof data === "string"
      ? new TextEncoder().encode(data)
      : data;

    this.pendingChunks.push(chunk);
    this.bufferedBytes += chunk.length;

    this.scheduleRender();

    // Check if we should pause
    if (!this.isPaused && this.bufferedBytes >= this.HIGH_WATERMARK) {
      this.isPaused = true;
      this.onPauseCallback?.();
    }

    return !this.isPaused;
  }

  /**
   * Write data directly to terminal (bypass flow control)
   * Use for small, critical data like user input echo
   */
  writeImmediate(data: string | Uint8Array): void {
    this.terminal?.write(data);
  }

  /**
   * Schedule batched render on next animation frame
   */
  private scheduleRender(): void {
    if (this.renderScheduled || this.pendingChunks.length === 0) return;

    this.renderScheduled = true;

    requestAnimationFrame(() => {
      this.renderScheduled = false;
      this.flushBatch();
    });
  }

  /**
   * Flush one batch of pending chunks
   */
  private flushBatch(): void {
    if (!this.terminal || this.pendingChunks.length === 0) return;

    let batchSize = 0;
    const chunksToWrite: Uint8Array[] = [];

    // Collect chunks up to MAX_BATCH_SIZE
    while (this.pendingChunks.length > 0 && batchSize < this.MAX_BATCH_SIZE) {
      const chunk = this.pendingChunks[0];
      if (batchSize + chunk.length > this.MAX_BATCH_SIZE && chunksToWrite.length > 0) {
        break;
      }
      chunksToWrite.push(this.pendingChunks.shift()!);
      batchSize += chunk.length;
    }

    if (chunksToWrite.length === 0) return;

    // Merge chunks for single write
    const merged = this.mergeChunks(chunksToWrite);

    // Write with callback for flow control
    this.terminal.write(merged, () => {
      this.bufferedBytes -= batchSize;
      this.totalBytesWritten += batchSize;
      this.writeCount++;

      // Check if we can resume
      if (this.isPaused && this.bufferedBytes < this.LOW_WATERMARK) {
        this.isPaused = false;
        this.onResumeCallback?.();
      }

      // Schedule next batch if more data pending
      if (this.pendingChunks.length > 0) {
        this.scheduleRender();
      }
    });
  }

  /**
   * Merge multiple chunks into single Uint8Array
   */
  private mergeChunks(chunks: Uint8Array[]): Uint8Array {
    if (chunks.length === 1) return chunks[0];

    const totalLength = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
    const merged = new Uint8Array(totalLength);

    let offset = 0;
    for (const chunk of chunks) {
      merged.set(chunk, offset);
      offset += chunk.length;
    }

    return merged;
  }

  /**
   * Immediately flush all pending data
   */
  flush(): void {
    if (!this.terminal) return;

    while (this.pendingChunks.length > 0) {
      const chunk = this.pendingChunks.shift()!;
      this.terminal.write(chunk);
      this.bufferedBytes -= chunk.length;
      this.totalBytesWritten += chunk.length;
      this.writeCount++;
    }

    this.isPaused = false;
  }

  /**
   * Check if flow control is currently pausing
   */
  shouldPause(): boolean {
    return this.isPaused;
  }

  /**
   * Get current flow control statistics
   */
  getStats(): FlowStats {
    return {
      bufferedBytes: this.bufferedBytes,
      pendingChunks: this.pendingChunks.length,
      isPaused: this.isPaused,
      totalBytesWritten: this.totalBytesWritten,
      writeCount: this.writeCount,
    };
  }

  /**
   * Reset statistics
   */
  resetStats(): void {
    this.totalBytesWritten = 0;
    this.writeCount = 0;
  }

  /**
   * Clear all pending data without writing
   */
  clear(): void {
    this.pendingChunks = [];
    this.bufferedBytes = 0;
    this.isPaused = false;
    this.renderScheduled = false;
  }
}
