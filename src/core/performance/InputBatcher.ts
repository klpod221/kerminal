/**
 * Input Batcher for Terminal Operations
 * Batches rapid terminal input to reduce Tauri invoke API calls
 */

import { writeToTerminal, writeBatchToTerminal } from "../../services/terminal";

/**
 * Batches terminal input to improve performance by reducing API calls
 */
export class InputBatcher {
  private static instance: InputBatcher | null = null;

  private pendingData: Map<string, string> = new Map();
  private timeouts: Map<string, number> = new Map();
  private readonly BATCH_DELAY = 16; // ~60fps, optimal for human typing

  /**
   * Get singleton instance
   */
  static getInstance(): InputBatcher {
    if (!InputBatcher.instance) {
      InputBatcher.instance = new InputBatcher();
    }
    return InputBatcher.instance;
  }

  /**
   * Add input data to batch for a specific terminal
   * @param terminalId - Backend terminal ID
   * @param data - Input data to send
   */
  public batchInput(terminalId: string, data: string): void {
    if (!terminalId || !data) return;

    // Append to existing pending data
    const currentData = this.pendingData.get(terminalId) || "";
    this.pendingData.set(terminalId, currentData + data);

    // Clear existing timeout if any
    const existingTimeout = this.timeouts.get(terminalId);
    if (existingTimeout) {
      clearTimeout(existingTimeout);
    }

    // Set new timeout to flush the batch
    const timeout = window.setTimeout(() => {
      this.flushInput(terminalId);
    }, this.BATCH_DELAY);

    this.timeouts.set(terminalId, timeout);
  }

  /**
   * Immediately flush pending input for a terminal
   * @param terminalId - Backend terminal ID
   */
  public async flushInput(terminalId: string): Promise<void> {
    const data = this.pendingData.get(terminalId);
    if (!data) return;

    // Clear pending data and timeout
    this.pendingData.delete(terminalId);
    const timeout = this.timeouts.get(terminalId);
    if (timeout) {
      clearTimeout(timeout);
      this.timeouts.delete(terminalId);
    }

    // Send the batched data
    try {
      await writeToTerminal({
        terminalId: terminalId,
        data,
      });
    } catch (error) {
      console.error(
        `Failed to send batched input to terminal ${terminalId}:`,
        error,
      );
    }
  }

  /**
   * Flush all pending inputs immediately
   * Uses batch API if multiple terminals have pending data
   */
  public async flushAllInputs(): Promise<void> {
    const terminalIds = Array.from(this.pendingData.keys());
    if (terminalIds.length === 0) return;

    try {
      if (terminalIds.length === 1) {
        // Single terminal - use regular flush
        await this.flushInput(terminalIds[0]);
      } else {
        // Multiple terminals - use batch API for better performance
        const requests = terminalIds
          .map((terminalId) => {
            const data = this.pendingData.get(terminalId);
            if (!data) return null;
            return {
              terminalId: terminalId,
              data,
            };
          })
          .filter(Boolean) as Array<{ terminalId: string; data: string }>;

        if (requests.length > 0) {
          // Clear all pending data and timeouts first
          terminalIds.forEach((terminalId) => {
            this.pendingData.delete(terminalId);
            const timeout = this.timeouts.get(terminalId);
            if (timeout) {
              clearTimeout(timeout);
              this.timeouts.delete(terminalId);
            }
          });

          // Send batch request
          await writeBatchToTerminal(requests);
        }
      }
    } catch (error) {
      console.error("Failed to flush all inputs:", error);
    }
  }

  /**
   * Check if terminal has pending input
   * @param terminalId - Backend terminal ID
   * @returns Whether terminal has pending input
   */
  public hasPendingInput(terminalId: string): boolean {
    return (
      this.pendingData.has(terminalId) &&
      this.pendingData.get(terminalId)!.length > 0
    );
  }

  /**
   * Get statistics about pending inputs
   */
  public getStats(): {
    pendingTerminals: number;
    totalPendingBytes: number;
    pendingTerminalIds: string[];
  } {
    const pendingTerminalIds = Array.from(this.pendingData.keys());
    const totalPendingBytes = pendingTerminalIds.reduce((total, terminalId) => {
      return total + (this.pendingData.get(terminalId)?.length || 0);
    }, 0);

    return {
      pendingTerminals: pendingTerminalIds.length,
      totalPendingBytes,
      pendingTerminalIds,
    };
  }

  /**
   * Clear all pending data and timeouts for a specific terminal
   * Useful when terminal is closed or disconnected
   * @param terminalId - Backend terminal ID
   */
  public clearTerminal(terminalId: string): void {
    this.pendingData.delete(terminalId);

    const timeout = this.timeouts.get(terminalId);
    if (timeout) {
      clearTimeout(timeout);
      this.timeouts.delete(terminalId);
    }
  }

  /**
   * Clear all pending data and timeouts
   */
  public clearAll(): void {
    // Clear all timeouts
    this.timeouts.forEach((timeout) => clearTimeout(timeout));

    // Clear all data
    this.pendingData.clear();
    this.timeouts.clear();
  }

  /**
   * Cleanup method for component unmounting
   */
  public cleanup(): void {
    this.clearAll();
  }
}
