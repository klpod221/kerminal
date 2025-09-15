export interface Tab {
  id: string
  title: string
  color?: string
  lastConnected?: Date
  profileId?: string
  groupId?: string
}

export interface Panel {
  id: string
  activeTabId: string
  tabs: Tab[]
}

export interface PanelLayout {
  type: 'panel' | 'split'
  id: string
  panel?: Panel // if type = 'panel'
  children?: PanelLayout[] // if type = 'split'
  direction?: 'horizontal' | 'vertical' // if type = 'split'
  sizes?: number[] // size ratios for children
}

export interface TerminalInstance {
  id: string
  ready: boolean
  isSSHConnecting?: boolean
  backendTerminalId?: string // ID of the terminal in the backend
  shouldFocusOnReady?: boolean // Flag to focus terminal when it becomes ready
  isClosing?: boolean // Flag to prevent double close operations
}

// Backend terminal types
export type TerminalType = 'Local' | 'SSH'

export type TerminalState = 'Connecting' | 'Connected' | 'Disconnected' | { Error: string }

export interface LocalConfig {
  shell?: string
  working_dir?: string
  env_vars?: Record<string, string>
}

export interface SSHConfig {
  host: string
  port: number
  username: string
  password?: string
  private_key_path?: string
  private_key_passphrase?: string
}

export interface TerminalConfig {
  terminal_type: TerminalType
  local_config?: LocalConfig
  ssh_config?: SSHConfig
}

export interface TerminalInfo {
  id: string
  config: TerminalConfig
  state: TerminalState
  created_at: string
  title?: string
}

export interface CreateTerminalRequest {
  config: TerminalConfig
  title?: string
}

export interface CreateTerminalResponse {
  terminal_id: string
  info: TerminalInfo
}

export interface WriteTerminalRequest {
  terminal_id: string
  data: string
}

export interface ResizeTerminalRequest {
  terminal_id: string
  cols: number
  rows: number
}

export interface TerminalData {
  terminal_id: string
  data: number[] // Vec<u8> from Rust
}

export interface TerminalTitleChanged {
  terminal_id: string
  title: string
}

export interface TerminalExited {
  terminal_id: string
  exit_code?: number
}
