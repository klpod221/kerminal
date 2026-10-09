// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

export interface TerminalProfile {
  id: string;
  name: string;
  shell: string;
  workingDir?: string;
  env?: Record<string, string>;
  icon?: string;
  color?: string;
  command?: string;
  isDefault?: boolean;
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
