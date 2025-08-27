/**
 * Interface for terminal instances and terminal management
 */

export interface ITerminalInstance {
  id: string
  ready: boolean
  type?: 'local' | 'ssh'
  metadata?: Record<string, unknown>
}

export interface ITerminalManager {
  createTerminal(terminalId: string): void
  createSSHTerminal(
    terminalId: string,
    config: unknown,
    profileId: string,
    profileName: string
  ): Promise<void>
  writeToTerminal(terminalId: string, data: string): void
  resizeTerminal(terminalId: string, cols: number, rows: number): void
  destroyTerminal(terminalId: string): void
  destroyAll(): void
  isSSHTerminal(terminalId: string): boolean
}

export interface IWindowManager {
  createWindow(): unknown
  minimize(): void
  toggleMaximize(): void
  close(): void
  isMaximized(): boolean
}

export interface IStorageService<T> {
  getAll(): Promise<T[]>
  getById(id: string): Promise<T | null>
  create(data: Omit<T, 'id' | 'created' | 'updated'>): Promise<T>
  update(id: string, updates: Partial<T>): Promise<T | null>
  delete(id: string): Promise<boolean>
}
