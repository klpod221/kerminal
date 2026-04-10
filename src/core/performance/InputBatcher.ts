/*
 * Kerminal - Modern Terminal Emulator & SSH Manager
 * Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

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

  private readonly pendingData: Map<string, string> = new Map();
  private readonly timeouts: Map<string, number> = new Map();
  private readonly BATCH_DELAY = 16; // ~60fps, optimal for human typing

  /**
   * Get singleton instance
   */
  static getInstance(): InputBatcher {
    InputBatcher.instance ??= new InputBatcher();
    return InputBatcher.instance;
  }

  /**
   * Add input data to batch for a specific terminal
   * @param terminalId - Backend terminal ID
   * @param data - Input data to send
   */
  public batchInput(terminalId: string, data: string): void {
    if (!terminalId || !data) return;

    const currentData = this.pendingData.get(terminalId) || "";
    this.pendingData.set(terminalId, currentData + data);

    const existingTimeout = this.timeouts.get(terminalId);
    if (existingTimeout) {
      clearTimeout(existingTimeout);
    }

    const timeout = globalThis.setTimeout(() => {
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

    this.pendingData.delete(terminalId);
    const timeout = this.timeouts.get(terminalId);
    if (timeout) {
      clearTimeout(timeout);
      this.timeouts.delete(terminalId);
    }

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
        await this.flushInput(terminalIds[0]);
      } else {
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
          terminalIds.forEach((terminalId) => {
            this.pendingData.delete(terminalId);
            const timeout = this.timeouts.get(terminalId);
            if (timeout) {
              clearTimeout(timeout);
              this.timeouts.delete(terminalId);
            }
          });

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
    this.timeouts.forEach((timeout) => clearTimeout(timeout));

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
