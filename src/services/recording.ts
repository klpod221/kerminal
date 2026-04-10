/*
 * Kerminal - Modern Terminal Emulator & SSH Manager
 * Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

import { api } from "./api";
import type { SessionRecording } from "../types/recording";

export async function startRecording(
  terminalId: string,
  sessionName?: string,
  width?: number,
  height?: number,
): Promise<string> {
  return await api.call("start_recording", {
    terminalId,
    sessionName,
    width,
    height,
  });
}

export async function stopRecording(
  terminalId: string,
): Promise<SessionRecording> {
  return await api.call("stop_recording", { terminalId });
}

export async function listRecordings(): Promise<SessionRecording[]> {
  return await api.call("list_recordings");
}

export async function deleteRecording(recordingId: string): Promise<void> {
  return await api.call("delete_recording", { recordingId });
}

export async function exportRecording(
  recordingId: string,
  exportPath: string,
): Promise<string> {
  return await api.call("export_recording", { recordingId, exportPath });
}

export async function readCastFile(filePath: string): Promise<string> {
  return await api.call("read_cast_file", { filePath });
}
