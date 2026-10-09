// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

export interface CommandHistoryEntry {
  command: string;
  timestamp?: string;
  index: number;
}

export interface GetTerminalHistoryRequest {
  terminalId: string;
  limit?: number;
}

export interface SearchHistoryRequest {
  terminalId: string;
  query: string;
  limit?: number;
}

export interface SearchHistoryResponse {
  entries: CommandHistoryEntry[];
  totalCount: number;
}

export interface ExportHistoryRequest {
  terminalId: string;
  format: "json" | "txt";
  filePath: string;
  query?: string;
}
