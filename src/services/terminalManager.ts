import type { TerminalTitleChanged, TerminalExited } from "../types/panel";
import { invokeWithErrorHandling, listenWithErrorHandling } from "../utils/terminalUtils";
import type {
  CreateTerminalRequest,
  CreateTerminalResponse,
  WriteTerminalRequest,
  ResizeTerminalRequest,
  TerminalInfo,
  TerminalData,
} from "../types/panel";

export class TerminalManager {
  private outputUnlisten: (() => void) | null = null;
  private titleUnlisten: (() => void) | null = null;
  private exitUnlisten: (() => void) | null = null;

  /**
   * Create a new terminal (local or SSH)
   */
  async createTerminal(request: CreateTerminalRequest): Promise<CreateTerminalResponse> {
    return invokeWithErrorHandling<CreateTerminalResponse>(
      "create_terminal",
      { request },
      "create terminal"
    );
  }

  /**
   * Write data to a terminal
   */
  async writeToTerminal(request: WriteTerminalRequest): Promise<void> {
    return invokeWithErrorHandling<void>(
      "write_to_terminal",
      { request },
      "write to terminal"
    );
  }

  /**
   * Resize a terminal
   */
  async resizeTerminal(request: ResizeTerminalRequest): Promise<void> {
    return invokeWithErrorHandling<void>(
      "resize_terminal",
      { request },
      "resize terminal"
    );
  }

  /**
   * Close a specific terminal
   */
  async closeTerminal(terminalId: string): Promise<void> {
    if (!terminalId || terminalId.trim() === '') {
      throw new Error('Terminal ID is required and cannot be empty');
    }

    return invokeWithErrorHandling<void>(
      "close_terminal",
      { terminalId },
      "close terminal"
    );
  }

  /**
   * Get information about a specific terminal
   */
  async getTerminalInfo(terminalId: string): Promise<TerminalInfo> {
    return invokeWithErrorHandling<TerminalInfo>(
      "get_terminal_info",
      { terminal_id: terminalId },
      "get terminal info"
    );
  }

  /**
   * List all active terminals
   */
  async listTerminals(): Promise<TerminalInfo[]> {
    return invokeWithErrorHandling<TerminalInfo[]>(
      "list_terminals",
      undefined,
      "list terminals"
    );
  }

  /**
   * Close all terminals
   */
  async closeAllTerminals(): Promise<void> {
    return invokeWithErrorHandling<void>(
      "close_all_terminals",
      undefined,
      "close all terminals"
    );
  }

  /**
   * Get user@hostname for terminal title
   */
  async getUserHostname(): Promise<string> {
    try {
      return await invokeWithErrorHandling<string>(
        "get_user_hostname",
        undefined,
        "get user hostname"
      );
    } catch {
      return "user@localhost";
    }
  }

  /**
   * Create a local terminal with default settings
   */
  async createLocalTerminal(title?: string): Promise<CreateTerminalResponse> {
    const request: CreateTerminalRequest = {
      config: {
        terminal_type: "Local",
        local_config: {},
      },
      title,
    };
    return this.createTerminal(request);
  }

  /**
   * Create an SSH terminal
   */
  async createSSHTerminal(
    host: string,
    port: number,
    username: string,
    password?: string,
    privateKeyPath?: string,
    title?: string
  ): Promise<CreateTerminalResponse> {
    const request: CreateTerminalRequest = {
      config: {
        terminal_type: "SSH",
        ssh_config: {
          host,
          port,
          username,
          password,
          private_key_path: privateKeyPath,
        },
      },
      title,
    };
    return this.createTerminal(request);
  }

  /**
   * Listen to terminal output events
   */
  async setupOutputListener(callback: (data: TerminalData) => void): Promise<() => void> {
    this.outputUnlisten = await listenWithErrorHandling<TerminalData>(
      "terminal-output",
      callback,
      "listen to terminal output"
    );
    return this.outputUnlisten;
  }

  /**
   * Listen to terminal title change events
   */
  async setupTitleListener(callback: (data: TerminalTitleChanged) => void): Promise<() => void> {
    this.titleUnlisten = await listenWithErrorHandling<TerminalTitleChanged>(
      "terminal-title-changed",
      callback,
      "listen to terminal title changes"
    );
    return this.titleUnlisten;
  }

  /**
   * Listen to terminal exit events
   */
  async setupExitListener(callback: (data: TerminalExited) => void): Promise<() => void> {
    this.exitUnlisten = await listenWithErrorHandling<TerminalExited>(
      "terminal-exited",
      callback,
      "listen to terminal exit events"
    );
    return this.exitUnlisten;
  }

  /**
   * Clean up all listeners
   */
  cleanup(): void {
    if (this.outputUnlisten) {
      this.outputUnlisten();
      this.outputUnlisten = null;
    }
    if (this.titleUnlisten) {
      this.titleUnlisten();
      this.titleUnlisten = null;
    }
    if (this.exitUnlisten) {
      this.exitUnlisten();
      this.exitUnlisten = null;
    }
  }
}

// Create singleton instance
export const terminalManager = new TerminalManager();
