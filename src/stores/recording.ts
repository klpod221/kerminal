import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import * as recordingService from '../services/recording';
import { api } from '../services/api';
import { writeToTerminal, getTerminalInfo } from '../services/terminal';
import { useWorkspaceStore } from './workspace';
import type { SessionRecording } from '../types/recording';

export const useRecordingStore = defineStore('recording', () => {
  const recordings = ref<SessionRecording[]>([]);
  const activeRecordings = ref<Map<string, string>>(new Map());
  const isLoading = ref(false);

  const isRecording = computed(() => (terminalId: string) => {
    return activeRecordings.value.has(terminalId);
  });

  async function loadRecordings() {
    isLoading.value = true;
    try {
      recordings.value = await recordingService.listRecordings();
    } catch (error) {
      console.error('Failed to load recordings:', error);
    } finally {
      isLoading.value = false;
    }
  }

  async function startRecording(terminalId: string, name?: string, width?: number, height?: number) {
    try {
      const workspaceStore = useWorkspaceStore();
      const backendTerminalId = await getBackendTerminalId(terminalId, workspaceStore);
      const recordingId = await recordingService.startRecording(backendTerminalId, name, width, height);
      activeRecordings.value.set(terminalId, recordingId);

      await sendRecordingMessage(terminalId, workspaceStore, 'start', name || recordingId);
      return recordingId;
    } catch (error) {
      console.error('Failed to start recording:', error);
      throw error;
    }
  }

  async function getBackendTerminalId(terminalId: string, workspaceStore: ReturnType<typeof useWorkspaceStore>): Promise<string> {
    let terminal = workspaceStore.terminals.find((t) => t.id === terminalId);

    if (!terminal) {
      terminal = workspaceStore.terminals.find((t) => t.backendTerminalId === terminalId);
    }

    return terminal?.backendTerminalId || terminalId;
  }

  async function sendRecordingMessage(
    terminalId: string,
    workspaceStore: ReturnType<typeof useWorkspaceStore>,
    type: 'start' | 'stop',
    data?: string | SessionRecording
  ): Promise<void> {
    let terminal = workspaceStore.terminals.find((t) => t.id === terminalId);

    if (!terminal) {
      terminal = workspaceStore.terminals.find((t) => t.backendTerminalId === terminalId);
    }

    if (!terminal?.backendTerminalId || !terminal.ready) {
      return;
    }

    try {
      const terminalInfo = await getTerminalInfo(terminal.backendTerminalId);
      const shell = terminalInfo.config.localConfig?.shell || '';
      const isWindowsCmd = shell.toLowerCase().includes('cmd.exe');
      const isPowerShell = shell.toLowerCase().includes('powershell') || shell.toLowerCase().includes('pwsh');

      let command = '';
      if (type === 'start') {
        const sessionName = typeof data === 'string' ? data : 'Unknown';
        const escapedName = sessionName.replace(/'/g, "'\\''");

        if (isWindowsCmd) {
          command = `echo.\r\necho [Recording Started] Session: ${sessionName}\r\necho.\r\n`;
        } else if (isPowerShell) {
          command = `Write-Host "` + `\r\n[Recording Started] Session: ${sessionName.replace(/"/g, '`"')}\r\n` + `" -ForegroundColor Yellow\r\n`;
        } else {
          command = `printf '\\r\\n\\033[33m[Recording Started]\\033[0m Session: ${escapedName}\\r\\n'\n`;
        }
      } else if (type === 'stop' && typeof data === 'object') {
        const recording = data as SessionRecording;
        let duration = 'N/A';
        if (recording.durationMs) {
          const totalSeconds = Math.floor(recording.durationMs / 1000);
          const minutes = Math.floor(totalSeconds / 60);
          const seconds = totalSeconds % 60;
          duration = `${minutes}:${String(seconds).padStart(2, '0')}`;
        }
        const fileName = recording.filePath ? recording.filePath.split('/').pop() || recording.filePath : recording.sessionName || recording.id;

        if (isWindowsCmd) {
          command = `echo.\r\necho [Recording Stopped] Duration: ${duration}, Saved as: ${fileName}\r\necho.\r\n`;
        } else if (isPowerShell) {
          command = `Write-Host "` + `\r\n[Recording Stopped] Duration: ${duration}, Saved as: ${fileName.replace(/"/g, '`"')}\r\n` + `" -ForegroundColor Green\r\n`;
        } else {
          const escapedFileName = fileName.replace(/'/g, "'\\''");
          command = `printf '\\r\\n\\033[32m[Recording Stopped]\\033[0m Duration: ${duration}, Saved as: ${escapedFileName}\\r\\n'\n`;
        }
      }

      if (command) {
        await writeToTerminal({
          terminalId: terminal.backendTerminalId,
          data: command,
        });
      }
    } catch (err) {
      console.error(`[Recording] Failed to write ${type} message:`, err);
      try {
        let fallbackMessage = '';
        if (type === 'start') {
          const sessionName = typeof data === 'string' ? data : 'Unknown';
          fallbackMessage = `\r\n\x1b[33m[Recording Started]\x1b[0m Session: ${sessionName}\r\n`;
        } else if (type === 'stop' && typeof data === 'object') {
          const recording = data as SessionRecording;
          let duration = 'N/A';
          if (recording.durationMs) {
            const totalSeconds = Math.floor(recording.durationMs / 1000);
            const minutes = Math.floor(totalSeconds / 60);
            const seconds = totalSeconds % 60;
            duration = `${minutes}:${String(seconds).padStart(2, '0')}`;
          }
          const fileName = recording.filePath ? recording.filePath.split('/').pop() || recording.filePath : recording.sessionName || recording.id;
          fallbackMessage = `\r\n\x1b[32m[Recording Stopped]\x1b[0m Duration: ${duration}, Saved as: ${fileName}\r\n`;
        }
        if (fallbackMessage) {
          await writeToTerminal({
            terminalId: terminal.backendTerminalId,
            data: fallbackMessage,
          });
        }
      } catch (fallbackErr) {
        console.error(`[Recording] Fallback also failed:`, fallbackErr);
      }
    }
  }

  async function stopRecording(terminalId: string) {
    try {
      const workspaceStore = useWorkspaceStore();
      const backendTerminalId = await getBackendTerminalId(terminalId, workspaceStore);
      const recording = await recordingService.stopRecording(backendTerminalId);
      activeRecordings.value.delete(terminalId);
      recordings.value.unshift(recording);

      await sendRecordingMessage(terminalId, workspaceStore, 'stop', recording);
      return recording;
    } catch (error) {
      console.error('Failed to stop recording:', error);
      throw error;
    }
  }

  async function deleteRecording(recordingId: string) {
    try {
      await recordingService.deleteRecording(recordingId);
      recordings.value = recordings.value.filter(r => r.id !== recordingId);
    } catch (error) {
      console.error('Failed to delete recording:', error);
      throw error;
    }
  }

  async function exportRecording(recordingId: string, path: string) {
    try {
      return await recordingService.exportRecording(recordingId, path);
    } catch (error) {
      console.error('Failed to export recording:', error);
      throw error;
    }
  }

  const upsertRecording = (r: SessionRecording) => {
    if (!r?.id) return;
    const i = recordings.value.findIndex((x) => x?.id === r.id);
    if (i === -1) {
      recordings.value = [r, ...recordings.value];
    } else {
      recordings.value[i] = { ...recordings.value[i]!, ...r };
    }
  };

  const removeRecording = (id: string) => {
    recordings.value = recordings.value.filter((r) => r?.id !== id);
  };

  let unsubscribeRealtime: (() => void) | null = null;

  const startRealtime = async (): Promise<void> => {
    if (unsubscribeRealtime) return;
    try {
      const u1 = await api.listen<SessionRecording>(
        'recording_saved',
        (r) => upsertRecording(r),
      );
      const u2 = await api.listen<SessionRecording>(
        'recording_updated',
        (r) => upsertRecording(r),
      );
      const u3 = await api.listen<{ id: string }>(
        'recording_deleted',
        ({ id }) => removeRecording(id),
      );
      unsubscribeRealtime = () => {
        u1();
        u2();
        u3();
      };
    } catch (e) {
      console.error('Failed to subscribe recording realtime events:', e);
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

    startRealtime,
    stopRealtime,
  };
});

