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
      throw error;
    }
  }

  /**
   * Call a Tauri command without parameter wrapping
   * @param command - The Tauri command name
   * @param args - Individual arguments to pass to the command
   * @returns Promise with the result
   */
  async callRaw<T = any>(
    command: string,
    ...args: any[]
  ): Promise<T> {
    try {
      if (args.length === 0) {
        return await invoke<T>(command);
      } else if (args.length === 1 && typeof args[0] === 'object' && args[0] !== null && !Array.isArray(args[0])) {
        // If single object argument, pass it directly (for backward compatibility)
        return await invoke<T>(command, args[0]);
      } else {
        // For multiple arguments, we need to map them to Tauri command parameters
        // Tauri expects named parameters, so we'll create an object
        const params: Record<string, any> = {};

        // Map arguments to parameter names based on command
        // This is a simple mapping - you might want to make this more sophisticated
        const paramNames = this.getParameterNames(command);
        args.forEach((arg, index) => {
          const paramName = paramNames[index] || `arg${index}`;
          params[paramName] = arg;
        });

        console.log(`🔍 API Debug - Command: ${command}`, {
          args,
          paramNames,
          params
        });

        return await invoke<T>(command, params);
      }
    } catch (error) {
      throw error;
    }
  }

  /**
   * Get parameter names for a given command
   * @param command - The command name
   * @returns Array of parameter names
   */
  private getParameterNames(command: string): string[] {
    // Define parameter mappings for commands
    const commandParams: Record<string, string[]> = {
      'update_ssh_profile': ['id', 'request'],
      'update_ssh_group': ['id', 'request'],
      'delete_ssh_profile': ['id'],
      'delete_ssh_group': ['id', 'action'],
      'get_ssh_profile': ['id'],
      'get_ssh_group': ['id'],
      'move_profile_to_group': ['profile_id', 'group_id'],
      'duplicate_ssh_profile': ['id', 'new_name'],
      // Tunnel commands
      'get_tunnel': ['id'],
      'update_tunnel': ['id', 'request'],
      'delete_tunnel': ['id'],
      'start_tunnel': ['id'],
      'stop_tunnel': ['id'],
      'get_tunnel_status': ['id'],
      // Saved command commands
      'get_saved_command': ['id'],
      'update_saved_command': ['id', 'request'],
      'delete_saved_command': ['id'],
      'increment_command_usage': ['id'],
      'toggle_command_favorite': ['id'],
      'get_saved_command_group': ['id'],
      'update_saved_command_group': ['id', 'request'],
      'delete_saved_command_group': ['id'],
    };

    return commandParams[command] || [];
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
      throw error;
    }
  }
}

// Create and export the default API client instance
export const api = new ApiClient();

// Also export the class for creating custom instances if needed
export { ApiClient };
