import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

/**
 * Generic error handler for terminal operations
 */
export const handleTerminalError = (operation: string, error: any): void => {
  console.error(`Failed to ${operation}:`, error);
  throw error;
};

/**
 * Generic invoke wrapper with error handling
 */
export const invokeWithErrorHandling = async <T>(
  command: string,
  args?: Record<string, any>,
  operation?: string,
): Promise<T> => {
  try {
    const finalArgs = args || {};
    return await invoke<T>(command, finalArgs);
  } catch (error) {
    handleTerminalError(operation || command, error);
    throw error; // Re-throw after logging
  }
};

/**
 * Generic event listener wrapper with error handling
 */
export const listenWithErrorHandling = async <T>(
  eventName: string,
  callback: (data: T) => void,
  operation?: string,
): Promise<() => void> => {
  try {
    const unlisten = await listen<T>(eventName, (event) => {
      callback(event.payload);
    });
    return unlisten;
  } catch (error) {
    handleTerminalError(operation || `listen to ${eventName}`, error);
    throw error; // Re-throw after logging
  }
};

/**
 * Convert byte array to string (for terminal output)
 */
export const bytesToString = (bytes: number[]): string => {
  return new TextDecoder().decode(new Uint8Array(bytes));
};
