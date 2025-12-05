export interface TerminalProfile {
  id: string;
  name: string;
  shell: string;
  workingDir?: string;
  env?: Record<string, string>;
  icon?: string;
  color?: string;
  command?: string;
}

export interface CreateTerminalProfileRequest {
  name: string;
  shell: string;
  workingDir?: string;
  env?: Record<string, string>;
  icon?: string;
  color?: string;
  command?: string;
}

export interface UpdateTerminalProfileRequest {
  name?: string;
  shell?: string;
  workingDir?: string;
  env?: Record<string, string>;
  icon?: string;
  color?: string;
  command?: string;
}
