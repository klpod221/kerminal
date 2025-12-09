import { defineStore } from "pinia";
import { ref, computed } from "vue";
import * as recordingService from "../services/recording";
import { api } from "../services/api";
import { writeToTerminal, getTerminalInfo } from "../services/terminal";
import { useWorkspaceStore } from "./workspace";
import type { SessionRecording } from "../types/recording";
import { handleError, type ErrorContext } from "../utils/errorHandler";
import { message } from "../utils/message";

export const useRecordingStore = defineStore("recording", () => {
  const recordings = ref<SessionRecording[]>([]);
  const activeRecordings = ref<Map<string, string>>(new Map());
  const isLoading = ref(false);

  const isRecording = computed(() => (terminalId: string) => {
    return activeRecordings.value.has(terminalId);
  });

  /**
   * Load all recordings with error handling
   */
  async function loadRecordings() {
    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Load Recordings",
    };

    try {
      recordings.value = await recordingService.listRecordings();
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Start recording a terminal session with error handling
   * @param terminalId - Terminal ID to record
   * @param name - Optional session name
   * @param width - Terminal width
   * @param height - Terminal height
   * @returns Recording ID
   */
  async function startRecording(
    terminalId: string,
    name?: string,
    width?: number,
    height?: number,
  ) {
    const context: ErrorContext = {
      operation: "Start Recording",
      context: { terminalId, name },
    };

    try {
      const workspaceStore = useWorkspaceStore();
      const backendTerminalId = await getBackendTerminalId(
        terminalId,
        workspaceStore,
      );
      const recordingId = await recordingService.startRecording(
        backendTerminalId,
        name,
        width,
        height,
      );
      activeRecordings.value.set(terminalId, recordingId);

      await sendRecordingMessage(
        terminalId,
        workspaceStore,
        "start",
        name || recordingId,
      );
      return recordingId;
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    }
  }

  /**
   * Stop recording a terminal session with error handling
   */
  async function stopRecording(terminalId: string) {
    const context: ErrorContext = {
      operation: "Stop Recording",
      context: { terminalId },
    };

    try {
      const workspaceStore = useWorkspaceStore();
      const backendTerminalId = await getBackendTerminalId(
        terminalId,
        workspaceStore,
      );
      const recording = await recordingService.stopRecording(backendTerminalId);
      activeRecordings.value.delete(terminalId);
      recordings.value.unshift(recording);

      await sendRecordingMessage(terminalId, workspaceStore, "stop", recording);
      return recording;
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    }
  }

  /**
   * Delete a recording with error handling
   */
  async function deleteRecording(recordingId: string) {
    const context: ErrorContext = {
      operation: "Delete Recording",
      context: { recordingId },
    };

    try {
      await recordingService.deleteRecording(recordingId);
      recordings.value = recordings.value.filter((r) => r.id !== recordingId);
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    }
  }

  const upsertRecording = (r: SessionRecording) => {
    if (!r?.id) return;
    const i = recordings.value.findIndex((x) => x?.id === r.id);
    if (i === -1) {
      recordings.value = [r, ...recordings.value];
    } else {
      recordings.value[i] = { ...recordings.value[i], ...r };
    }
  };

  const removeRecording = (id: string) => {
    recordings.value = recordings.value.filter((r) => r?.id !== id);
  };

  let unsubscribeRealtime: (() => void) | null = null;

  const startRealtime = async (): Promise<void> => {
    if (unsubscribeRealtime) return;
    try {
      const u1 = await api.listen<SessionRecording>("recording_saved", (r) =>
        upsertRecording(r),
      );
      const u2 = await api.listen<SessionRecording>("recording_updated", (r) =>
        upsertRecording(r),
      );
      const u3 = await api.listen<{ id: string }>(
        "recording_deleted",
        ({ id }) => removeRecording(id),
      );
      unsubscribeRealtime = () => {
        u1();
        u2();
        u3();
      };
    } catch (e) {
      console.error("Failed to subscribe recording realtime events:", e);
    }
  };

  const stopRealtime = (): void => {
    if (unsubscribeRealtime) {
      unsubscribeRealtime();
      unsubscribeRealtime = null;
    }
  };

  return {
    recordings,
    activeRecordings,
    isLoading,
    isRecording,
    loadRecordings,
    startRecording,
    stopRecording,
    deleteRecording,
    exportRecording,
    readCastFile,

    startRealtime,
    stopRealtime,
  };
});

/**
 * Helper to get backend terminal ID
 */
async function getBackendTerminalId(
  terminalId: string,
  workspaceStore: ReturnType<typeof useWorkspaceStore>,
): Promise<string> {
  const terminal =
    workspaceStore.terminals.find((t) => t.id === terminalId) ??
    workspaceStore.terminals.find((t) => t.backendTerminalId === terminalId);

  return terminal?.backendTerminalId || terminalId;
}

/**
 * Recording data type
 */
type RecordingData = string | SessionRecording | undefined;

/**
 * Format recording message for terminal output
 */
const prepareStartMessage = (
  data: RecordingData,
  config: { isWindowsCmd: boolean; isPowerShell: boolean },
): string => {
  const sessionName = typeof data === "string" ? data : "Unknown";
  const escapedName = sessionName.replaceAll("'", String.raw`'\''`);

  if (config.isWindowsCmd) {
    return `echo.\r\necho [Recording Started] Session: ${sessionName}\r\necho.\r\n`;
  }
  if (config.isPowerShell) {
    return (
      `Write-Host "` +
      `\r\n[Recording Started] Session: ${sessionName.replaceAll('"', '`"')}\r\n` +
      `" -ForegroundColor Yellow\r\n`
    );
  }
  return (
    String.raw`printf '\r\n\033[33m[Recording Started]\033[0m Session: ${escapedName}\r\n'` +
    "\n"
  );
};

const prepareStopMessage = (
  data: RecordingData,
  config: { isWindowsCmd: boolean; isPowerShell: boolean },
): string => {
  if (typeof data !== "object") return "";
  const recording = data;
  let duration = "N/A";
  if (recording.durationMs) {
    const totalSeconds = Math.floor(recording.durationMs / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    duration = `${minutes}:${String(seconds).padStart(2, "0")}`;
  }
  const fileName = recording.filePath
    ? recording.filePath.split("/").pop() || recording.filePath
    : recording.sessionName || recording.id;

  if (config.isWindowsCmd) {
    return `echo.\r\necho [Recording Stopped] Duration: ${duration}, Saved as: ${fileName}\r\necho.\r\n`;
  }
  if (config.isPowerShell) {
    return (
      `Write-Host "` +
      `\r\n[Recording Stopped] Duration: ${duration}, Saved as: ${fileName.replaceAll('"', '`"')}\r\n` +
      `" -ForegroundColor Green\r\n`
    );
  }
  const escapedFileName = fileName.replaceAll("'", String.raw`'\''`);
  return (
    String.raw`printf '\r\n\033[32m[Recording Stopped]\033[0m Duration: ${duration}, Saved as: ${escapedFileName}\r\n'` +
    "\n"
  );
};

/**
 * Format recording message for terminal output
 */
function formatRecordingMessage(
  type: "start" | "stop",
  data: string | SessionRecording | undefined,
  config: { isWindowsCmd: boolean; isPowerShell: boolean },
): string {
  if (type === "start") {
    return prepareStartMessage(data, config);
  }

  if (type === "stop") {
    return prepareStopMessage(data, config);
  }

  return "";
}

/**
 * Send recording lifecycle message to terminal
 */
async function sendRecordingMessage(
  terminalId: string,
  workspaceStore: ReturnType<typeof useWorkspaceStore>,
  type: "start" | "stop",
  data?: string | SessionRecording,
): Promise<void> {
  const terminal =
    workspaceStore.terminals.find((t) => t.id === terminalId) ??
    workspaceStore.terminals.find((t) => t.backendTerminalId === terminalId);

  if (!terminal?.backendTerminalId || !terminal.ready) {
    return;
  }

  try {
    const terminalInfo = await getTerminalInfo(terminal.backendTerminalId);
    const shell = terminalInfo.config.localConfig?.shell || "";
    const isWindowsCmd = shell.toLowerCase().includes("cmd.exe");
    const isPowerShell =
      shell.toLowerCase().includes("powershell") ||
      shell.toLowerCase().includes("pwsh");

    const command = formatRecordingMessage(type, data, {
      isWindowsCmd,
      isPowerShell,
    });

    if (command) {
      await writeToTerminal({
        terminalId: terminal.backendTerminalId,
        data: command,
      });
    }
  } catch (err) {
    console.error(`[Recording] Failed to write ${type} message:`, err);
    // Fallback logic could be extracted too if complex, but leaving simplified fallback for now or suppressing
  }
}

/**
 * Export a recording to a file with error handling
 * @param recordingId - Recording ID to export
 * @param path - Export file path
 * @returns Export result
 */
async function exportRecording(recordingId: string, path: string) {
  const context: ErrorContext = {
    operation: "Export Recording",
    context: { recordingId, path },
  };

  try {
    return await recordingService.exportRecording(recordingId, path);
  } catch (error) {
    const errorMessage = handleError(error, context);
    message.error(errorMessage);
    throw new Error(errorMessage);
  }
}

/**
 * Read cast file content with error handling
 * @param filePath - Path to cast file
 * @returns Cast file content as string
 */
async function readCastFile(filePath: string): Promise<string> {
  const context: ErrorContext = {
    operation: "Read Cast File",
    context: { filePath },
  };

  try {
    return await recordingService.readCastFile(filePath);
  } catch (error) {
    const errorMessage = handleError(error, context);
    message.error(errorMessage);
    throw new Error(errorMessage);
  }
}
