import { api } from "./api";
import { terminalCache } from "../core/performance";
import type {
  TerminalTitleChanged,
  TerminalExited,
  CreateTerminalResponse,
  WriteTerminalRequest,
  ResizeTerminalRequest,
  TerminalInfo,
  TerminalData,
  TerminalLatency,
} from "../types/panel";

let outputUnlisten: (() => void) | null = null;
let titleUnlisten: (() => void) | null = null;
let exitUnlisten: (() => void) | null = null;
let latencyUnlisten: (() => void) | null = null;

/**
 * Create a new local terminal
 */
export async function createTerminal(
  shell?: string,
  workingDir?: string,
  title?: string,
): Promise<CreateTerminalResponse> {
  try {
    return await api.call<CreateTerminalResponse>("create_terminal", {
      shell,
      workingDir,
      title,
    });
  } catch (error) {
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
    return await api.call<CreateTerminalResponse>("create_ssh_terminal", {
      profileId,
    });
  } catch (error) {
    throw error;
  }
}

/**
 * Create a new SSH terminal from SSH config host
 */
export async function createSSHConfigTerminal(
  hostName: string,
  title?: string,
  password?: string,
): Promise<CreateTerminalResponse> {
  try {
    return await api.call<CreateTerminalResponse>(
      "create_ssh_config_terminal",
      {
        hostName,
        title,
        password,
      },
    );
  } catch (error) {
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
    return await api.call<void>("write_to_terminal", request);
  } catch (error) {
    throw error;
  }
}

/**
 * Write data to multiple terminals in batch
 */
export async function writeBatchToTerminal(
  requests: WriteTerminalRequest[],
): Promise<void> {
  try {
    return await api.call<void>("write_batch_to_terminal", {
      requests,
    });
  } catch (error) {
    console.error("Failed to write batch to terminals:", error);
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
    return await api.call<void>("resize_terminal", request);
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
    const result = await api.call<void>("close_terminal", {
      terminalId,
    });

    terminalCache.invalidateTerminal(terminalId);

    return result;
  } catch (error) {
    console.error("Failed to close terminal:", error);
    throw error;
  }
}

/**
 * Get information about a specific terminal (cached)
 */
export async function getTerminalInfo(
  terminalId: string,
): Promise<TerminalInfo> {
  try {
    return await terminalCache.getTerminalInfo(terminalId);
  } catch (error) {
    console.error("Failed to get terminal info:", error);
    throw error;
  }
}

/**
 * List all active terminals (cached)
 */
export async function listTerminals(): Promise<TerminalInfo[]> {
  try {
    return await terminalCache.getTerminalList();
  } catch (error) {
    console.error("Failed to list terminals:", error);
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
  outputUnlisten = await api.listen<TerminalData>("terminal-output", callback);
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
  exitUnlisten = await api.listen<TerminalExited>("terminal-exited", callback);
  return exitUnlisten;
}

/**
 * Listen to terminal latency events
 */
export async function listenToTerminalLatency(
  callback: (data: TerminalLatency) => void,
): Promise<() => void> {
  latencyUnlisten = await api.listen<TerminalLatency>(
    "terminal-latency",
    callback,
  );
  return latencyUnlisten;
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
  if (latencyUnlisten) {
    latencyUnlisten();
    latencyUnlisten = null;
  }
};
