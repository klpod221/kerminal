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
  async callRaw<T = any>(command: string, ...args: any[]): Promise<T> {
    try {
      if (args.length === 0) {
        return await invoke<T>(command);
      } else if (
        args.length === 1 &&
        typeof args[0] === "object" &&
        args[0] !== null &&
        !Array.isArray(args[0])
      ) {
        return await invoke<T>(command, args[0]);
      } else {
        const params: Record<string, any> = {};

        const paramNames = this.getParameterNames(command);
        args.forEach((arg, index) => {
          const paramName = paramNames[index] || `arg${index}`;
          params[paramName] = arg;
        });

      // Debug logging (disabled in production)
      if (import.meta.env.DEV) {
        console.log(`🔍 API Debug - Command: ${command}`, {
          args,
          paramNames,
          params,
        });
      }

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
    const commandParams: Record<string, string[]> = {
      update_ssh_profile: ["id", "request"],
      update_ssh_group: ["id", "request"],
      delete_ssh_profile: ["id"],
      delete_ssh_group: ["id", "action"],
      get_ssh_profile: ["id"],
      get_ssh_group: ["id"],
      move_profile_to_group: ["profile_id", "group_id"],
      duplicate_ssh_profile: ["id", "new_name"],
      get_tunnel: ["id"],
      update_tunnel: ["id", "request"],
      delete_tunnel: ["id"],
      start_tunnel: ["id"],
      stop_tunnel: ["id"],
      get_tunnel_status: ["id"],
      get_saved_command: ["id"],
      update_saved_command: ["id", "request"],
      delete_saved_command: ["id"],
      increment_command_usage: ["id"],
      toggle_command_favorite: ["id"],
      get_saved_command_group: ["id"],
      update_saved_command_group: ["id", "request"],
      delete_saved_command_group: ["id"],
      sftp_connect: ["profileId"],
      sftp_disconnect: ["sessionId"],
      sftp_list_directory: ["sessionId", "path"],
      sftp_stat: ["sessionId", "path"],
      sftp_create_directory: ["sessionId", "path"],
      sftp_rename: ["sessionId", "oldPath", "newPath"],
      sftp_delete: ["sessionId", "path", "recursive"],
      sftp_set_permissions: ["sessionId", "path", "mode"],
      sftp_create_symlink: ["sessionId", "target", "linkPath"],
      sftp_read_symlink: ["sessionId", "path"],
      sftp_upload_file: ["sessionId", "localPath", "remotePath"],
      sftp_download_file: ["sessionId", "remotePath", "localPath"],
      sftp_get_transfer_progress: ["transferId"],
      sftp_cancel_transfer: ["transferId"],
      sftp_resume_transfer: ["transferId"],
      sftp_compare_directories: ["sessionId", "localPath", "remotePath"],
      sftp_sync_directory: ["sessionId", "operation"],
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

export const api = new ApiClient();

export { ApiClient };
