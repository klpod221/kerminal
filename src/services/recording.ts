import { api } from './api';
import type { SessionRecording } from '../types/recording';

export async function startRecording(
  terminalId: string,
  sessionName?: string,
  width?: number,
  height?: number
): Promise<string> {
  return await api.call('start_recording', { 
    terminalId, 
    sessionName,
    width,
    height 
  });
}

export async function stopRecording(
  terminalId: string
): Promise<SessionRecording> {
  return await api.call('stop_recording', { terminalId });
}

export async function listRecordings(): Promise<SessionRecording[]> {
  return await api.call('list_recordings');
}

export async function deleteRecording(recordingId: string): Promise<void> {
  return await api.call('delete_recording', { recordingId });
}

export async function exportRecording(
  recordingId: string,
  exportPath: string
): Promise<string> {
  return await api.call('export_recording', { recordingId, exportPath });
}

