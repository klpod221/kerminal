import type { TerminalTitleChanged, TerminalExited } from "../types/panel";
import { api } from "./api";
import type {
  CreateTerminalResponse,
  WriteTerminalRequest,
  ResizeTerminalRequest,
  TerminalInfo,
  TerminalData,
} from "../types/panel";

// Terminal service listeners for cleanup
let outputUnlisten: (() => void) | null = null;
let titleUnlisten: (() => void) | null = null;
let exitUnlisten: (() => void) | null = null;

/**
 * Create a new local terminal
 */
export async function createTerminal(
  shell?: string,
  workingDir?: string,
  title?: string,
): Promise<CreateTerminalResponse> {
  try {
    return await api.call<CreateTerminalResponse>(
      "create_terminal",
      {
        shell,
        working_dir: workingDir,
        title,
      },
    );
  } catch (error) {
    console.error("Failed to create terminal:", error);
    throw error;
  }
}

/**
 * Create a new local terminal (alias for createTerminal)
 */
export const createLocalTerminal = createTerminal;

/**
 * Create a new SSH terminal using profile ID
 */
export async function createSSHTerminal(
  profileId: string,
): Promise<CreateTerminalResponse> {
  try {
    return await api.call<CreateTerminalResponse>(
      "create_ssh_terminal",
      {
        profile_id: profileId,
      },
    );
  } catch (error) {
    console.error("Failed to create SSH terminal:", error);
    throw error;
  }
}

/**
 * Write data to a terminal
 */
export async function writeToTerminal(
  request: WriteTerminalRequest,
): Promise<void> {
  try {
    return await api.call<void>(
      "write_to_terminal",
      {
        terminal_id: request.terminal_id,
        data: request.data,
      },
    );
  } catch (error) {
    console.error("Failed to write to terminal:", error);
    throw error;
  }
}

/**
 * Resize a terminal
 */
export async function resizeTerminal(
  request: ResizeTerminalRequest,
): Promise<void> {
  try {
    return await api.call<void>(
      "resize_terminal",
      request,
    );
  } catch (error) {
    console.error("Failed to resize terminal:", error);
    throw error;
  }
}

/**
 * Close a specific terminal
 */
export async function closeTerminal(terminalId: string): Promise<void> {
  if (!terminalId || terminalId.trim() === "") {
    throw new Error("Terminal ID is required and cannot be empty");
  }

  try {
    return await api.call<void>(
      "close_terminal",
      { terminal_id: terminalId },
    );
  } catch (error) {
    console.error("Failed to close terminal:", error);
    throw error;
  }
}

/**
 * Get information about a specific terminal
 */
export async function getTerminalInfo(
  terminalId: string,
): Promise<TerminalInfo> {
  try {
    return await api.call<TerminalInfo>(
      "get_terminal_info",
      { terminal_id: terminalId },
    );
  } catch (error) {
    console.error("Failed to get terminal info:", error);
    throw error;
  }
}

/**
 * List all active terminals
 */
export async function listTerminals(): Promise<TerminalInfo[]> {
  try {
    return await api.callRaw<TerminalInfo[]>("list_terminals");
  } catch (error) {
    console.error("Failed to list terminals:", error);
    throw error;
  }
}

/**
 * Close all terminals
 */
export async function closeAllTerminals(): Promise<void> {
  try {
    return await api.callRaw<void>("close_all_terminals");
  } catch (error) {
    console.error("Failed to close all terminals:", error);
    throw error;
  }
}

/**
 * Get user@hostname for terminal title
 */
export async function getUserHostname(): Promise<string> {
  try {
    return await api.callRaw<string>("get_user_hostname");
  } catch (error) {
    console.error("Failed to get user hostname:", error);
    return "user@hostname"; // fallback
  }
}

/**
 * Listen to terminal output events
 */
export async function listenToTerminalOutput(
  callback: (data: TerminalData) => void,
): Promise<() => void> {
  outputUnlisten = await api.listen<TerminalData>(
    "terminal-output",
    callback,
  );
  return outputUnlisten;
}

/**
 * Listen to terminal title change events
 */
export async function listenToTerminalTitleChanged(
  callback: (data: TerminalTitleChanged) => void,
): Promise<() => void> {
  titleUnlisten = await api.listen<TerminalTitleChanged>(
    "terminal-title-changed",
    callback,
  );
  return titleUnlisten;
}

/**
 * Listen to terminal exit events
 */
export async function listenToTerminalExit(
  callback: (data: TerminalExited) => void,
): Promise<() => void> {
  exitUnlisten = await api.listen<TerminalExited>(
    "terminal-exited",
    callback,
  );
  return exitUnlisten;
}

/**
 * Cleanup all terminal listeners
 */
export const cleanupTerminalListeners = (): void => {
  if (outputUnlisten) {
    outputUnlisten();
    outputUnlisten = null;
  }
  if (titleUnlisten) {
    titleUnlisten();
    titleUnlisten = null;
  }
  if (exitUnlisten) {
    exitUnlisten();
    exitUnlisten = null;
  }
};
