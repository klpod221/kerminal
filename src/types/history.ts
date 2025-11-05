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
