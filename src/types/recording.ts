export interface SessionRecording {
  id: string;
  terminalId: string;
  sessionName: string;
  terminalType: "Local" | "SSH";
  startedAt: string;
  endedAt?: string;
  durationMs?: number;
  filePath: string;
  fileSize: number;
  width: number;
  height: number;
  metadata?: string;
  createdAt: string;
}

export interface RecordingMetadata {
  tags: string[];
  description?: string;
  sshHost?: string;
  shell?: string;
}

export interface AsciicastHeader {
  version: number;
  width: number;
  height: number;
  timestamp?: number;
  title?: string;
  env?: Record<string, string>;
}

export interface AsciicastEvent {
  time: number;
  eventType: string;
  data: string;
}
