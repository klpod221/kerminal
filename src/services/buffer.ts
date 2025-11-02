import { api } from "./api";
import { terminalCache } from "../core/performance";

/**
 * Buffer statistics interface
 */
export interface BufferStats {
  totalTerminals: number;
  totalLines: number;
  memoryUsage: number;
}

/**
 * Get buffer as string from Rust backend
 * @param terminalId - Terminal identifier
 * @returns Promise of buffer string
 */
export async function getTerminalBuffer(terminalId: string): Promise<string> {
  return await api.call<string>("get_terminal_buffer", {
    terminalId,
  });
}

/**
 * Check if terminal has buffer in Rust backend (cached)
 * @param terminalId - Terminal identifier
 * @returns Promise of boolean
 */
export async function hasTerminalBuffer(terminalId: string): Promise<boolean> {
  return await terminalCache.hasTerminalBuffer(terminalId);
}

/**
 * Get buffer statistics from Rust backend (cached)
 * @returns Promise of buffer statistics
 */
export async function getBufferStats(): Promise<BufferStats> {
  const stats = await terminalCache.getBufferStats();
  return {
    totalTerminals: stats.totalTerminals || 0,
    totalLines: stats.totalLines || 0,
    memoryUsage: stats.memoryUsage || 0,
  };
}

/**
 * Cleanup orphaned buffers in Rust backend
 * @param activeTerminalIds - Array of active terminal IDs
 */
export async function cleanupTerminalBuffers(
  activeTerminalIds: string[],
): Promise<void> {
  return await api.call<void>("cleanup_terminal_buffers", {
    activeTerminalIds,
  });
}

/**
 * List all terminals from backend
 * @returns Promise of terminal list
 */
export async function listTerminals(): Promise<Array<{ id: string }>> {
  return await api.callRaw<Array<{ id: string }>>("list_terminals");
}
