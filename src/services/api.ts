import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

/**
 * API wrapper class similar to axios instance
 * Automatically wraps parameters and provides consistent interface
 */
class ApiClient {
  /**
   * Call a Tauri command with automatic parameter wrapping
   * @param command - The Tauri command name
   * @param data - The data to send (will be wrapped in { request: data })
   * @returns Promise with the result
   */
  async call<T = any>(command: string, data?: any): Promise<T> {
    try {
      if (data === undefined || data === null) {
        return await invoke<T>(command);
      }

      // Wrap data in request object for consistent backend handling
      const params = { request: data };
      return await invoke<T>(command, params);
    } catch (error) {
      console.error(`API call failed for command "${command}":`, error);
      throw error;
    }
  }

  /**
   * Call a Tauri command without parameter wrapping
   * @param command - The Tauri command name
   * @param params - Raw parameters to send directly
   * @returns Promise with the result
   */
  async callRaw<T = any>(
    command: string,
    params?: Record<string, any>,
  ): Promise<T> {
    try {
      return await invoke<T>(command, params);
    } catch (error) {
      console.error(`Raw API call failed for command "${command}":`, error);
      throw error;
    }
  }

  /**
   * Listen to a Tauri event with error handling
   * @param eventName - The event name to listen to
   * @param callback - Callback function to handle the event
   * @returns Promise that resolves to an unlisten function
   */
  async listen<T = any>(
    eventName: string,
    callback: (data: T) => void,
  ): Promise<() => void> {
    try {
      const unlisten = await listen<T>(eventName, (event) => {
        callback(event.payload);
      });
      return unlisten;
    } catch (error) {
      console.error(`Failed to listen to event "${eventName}":`, error);
      throw error;
    }
  }
}

// Create and export the default API client instance
export const api = new ApiClient();

// Also export the class for creating custom instances if needed
export { ApiClient };
