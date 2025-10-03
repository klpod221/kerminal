export interface Tab {
  id: string;
  title: string;
  color?: string;
  lastConnected?: Date;
  profileId?: string;
  groupId?: string;
}

export interface Panel {
  id: string;
  activeTabId: string;
  tabs: Tab[];
}

export interface PanelLayout {
  type: "panel" | "split";
  id: string;
  panel?: Panel; // if type = 'panel'
  children?: PanelLayout[]; // if type = 'split'
  direction?: "horizontal" | "vertical"; // if type = 'split'
  sizes?: number[]; // size ratios for children
}

export interface TerminalInstance {
  id: string;
  ready: boolean;
  isSSHConnecting?: boolean;
  backendTerminalId?: string; // ID of the terminal in the backend
  shouldFocusOnReady?: boolean; // Flag to focus terminal when it becomes ready
  isClosing?: boolean; // Flag to prevent double close operations
  disconnectReason?: "user-closed" | "connection-lost"; // Reason for disconnect
  canReconnect?: boolean; // Whether reconnect is available (for SSH)
  sshProfileId?: string; // SSH profile ID for reconnection
}

// Backend terminal types
export type TerminalType = "Local" | "SSH";

export type TerminalState =
  | "Connecting"
  | "Connected"
  | "Disconnected"
  | { Error: string };

export interface LocalConfig {
  shell?: string;
  workingDir?: string;
  envVars?: Record<string, string>;
}

export interface SSHConfig {
  host: string;
  port: number;
  username: string;
  password?: string;
  private_key_path?: string;
  private_key_passphrase?: string;
}

export interface TerminalConfig {
  terminalType: TerminalType;
  localConfig?: LocalConfig;
  sshConfig?: SSHConfig;
}

export interface TerminalInfo {
  id: string;
  config: TerminalConfig;
  state: TerminalState;
  createdAt: string;
  title?: string;
}

export interface CreateTerminalRequest {
  config: TerminalConfig;
  title?: string;
}

export interface CreateTerminalResponse {
  terminalId: string;
  info: TerminalInfo;
}

export interface WriteTerminalRequest {
  terminalId: string;
  data: string;
}

export interface ResizeTerminalRequest {
  terminalId: string;
  cols: number;
  rows: number;
}

export interface TerminalData {
  terminalId: string;
  data: number[]; // Vec<u8> from Rust
}

export interface TerminalTitleChanged {
  terminalId: string;
  title: string;
}

export interface TerminalExited {
  terminalId: string;
  exitCode?: number;
  reason?: "user-closed" | "connection-lost" | "error";
}
