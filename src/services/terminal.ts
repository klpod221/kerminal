import type { TerminalTitleChanged, TerminalExited } from "../types/panel";
import {
  invokeWithErrorHandling,
  listenWithErrorHandling,
} from "../utils/terminal";
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
  return invokeWithErrorHandling<CreateTerminalResponse>(
    "create_terminal",
    {
      shell,
      working_dir: workingDir,
      title,
    },
    "create terminal",
  );
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
  return invokeWithErrorHandling<CreateTerminalResponse>(
    "create_ssh_terminal",
    { profile_id: profileId },
    "create SSH terminal",
  );
}

/**
 * Write data to a terminal
 */
export async function writeToTerminal(
  request: WriteTerminalRequest,
): Promise<void> {
  return invokeWithErrorHandling<void>(
    "write_to_terminal",
    { request },
    "write to terminal",
  );
}

/**
 * Resize a terminal
 */
export async function resizeTerminal(
  request: ResizeTerminalRequest,
): Promise<void> {
  return invokeWithErrorHandling<void>(
    "resize_terminal",
    { request },
    "resize terminal",
  );
}

/**
 * Close a specific terminal
 */
export async function closeTerminal(terminalId: string): Promise<void> {
  if (!terminalId || terminalId.trim() === "") {
    throw new Error("Terminal ID is required and cannot be empty");
  }

  return invokeWithErrorHandling<void>(
    "close_terminal",
    { terminalId },
    "close terminal",
  );
}

/**
 * Get information about a specific terminal
 */
export async function getTerminalInfo(
  terminalId: string,
): Promise<TerminalInfo> {
  return invokeWithErrorHandling<TerminalInfo>(
    "get_terminal_info",
    { terminal_id: terminalId },
    "get terminal info",
  );
}

/**
 * List all active terminals
 */
export async function listTerminals(): Promise<TerminalInfo[]> {
  return invokeWithErrorHandling<TerminalInfo[]>(
    "list_terminals",
    undefined,
    "list terminals",
  );
}

/**
 * Close all terminals
 */
export async function closeAllTerminals(): Promise<void> {
  return invokeWithErrorHandling<void>(
    "close_all_terminals",
    undefined,
    "close all terminals",
  );
}

/**
 * Get user@hostname for terminal title
 */
export async function getUserHostname(): Promise<string> {
  try {
    return await invokeWithErrorHandling<string>(
      "get_user_hostname",
      undefined,
      "get user hostname",
    );
  } catch {
    return "user@localhost";
  }
}

/**
 * Listen to terminal output events
 */
export async function listenToTerminalOutput(
  callback: (data: TerminalData) => void,
): Promise<() => void> {
  outputUnlisten = await listenWithErrorHandling<TerminalData>(
    "terminal-output",
    callback,
    "listen to terminal output",
  );
  return outputUnlisten;
}

/**
 * Listen to terminal title change events
 */
export async function listenToTerminalTitleChanges(
  callback: (data: TerminalTitleChanged) => void,
): Promise<() => void> {
  titleUnlisten = await listenWithErrorHandling<TerminalTitleChanged>(
    "terminal-title-changed",
    callback,
    "listen to terminal title changes",
  );
  return titleUnlisten;
}

/**
 * Listen to terminal exit events
 */
export async function listenToTerminalExits(
  callback: (data: TerminalExited) => void,
): Promise<() => void> {
  exitUnlisten = await listenWithErrorHandling<TerminalExited>(
    "terminal-exited",
    callback,
    "listen to terminal exit events",
  );
  return exitUnlisten;
}

/**
 * Clean up all listeners
 */
export function cleanupTerminalListeners(): void {
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
}

// Re-export utility
export { bytesToString } from "../utils/terminal";
