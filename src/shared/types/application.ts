/**
 * Common application interfaces and types shared between main and renderer
 */

export interface IApplicationLifecycle {
  initialize(): Promise<void>
  cleanup(): void
}

export interface IService {
  name: string
  isInitialized(): boolean
  initialize?(): Promise<void>
  cleanup?(): void
}

export interface IEventEmitter<T = Record<string, unknown[]>> {
  on<K extends keyof T>(
    event: K,
    handler: (...args: T[K] extends unknown[] ? T[K] : never) => void
  ): () => void
  emit<K extends keyof T>(event: K, ...args: T[K] extends unknown[] ? T[K] : never): void
  off<K extends keyof T>(
    event: K,
    handler: (...args: T[K] extends unknown[] ? T[K] : never) => void
  ): void
}

export interface ILogger {
  info(message: string, ...args: unknown[]): void
  warn(message: string, ...args: unknown[]): void
  error(message: string, error?: Error, ...args: unknown[]): void
  debug(message: string, ...args: unknown[]): void
}

export interface IConfiguration {
  get<T = unknown>(key: string): T | undefined
  set<T = unknown>(key: string, value: T): void
  has(key: string): boolean
}

export interface IValidationResult {
  valid: boolean
  errors: string[]
  warnings?: string[]
}

export interface IValidator<T> {
  validate(data: T): IValidationResult
}

export interface IConnection extends IApplicationLifecycle {
  connect(): Promise<void>
  disconnect(): void
  isConnected(): boolean
}

export interface IPaginationOptions {
  page: number
  limit: number
  sortBy?: string
  sortOrder?: 'asc' | 'desc'
}

export interface IPaginationResult<T> {
  items: T[]
  total: number
  page: number
  totalPages: number
  hasNext: boolean
  hasPrev: boolean
}

/**
 * Window configuration interface
 */
export interface WindowConfig {
  width?: number
  height?: number
  show?: boolean
  autoHideMenuBar?: boolean
  frame?: boolean
}

/**
 * Context menu item interface
 */
export interface MenuItem {
  label: string
  enabled: boolean
  action?: () => void
}

/**
 * Network interface information
 */
export interface NetworkInterface {
  name: string
  address: string
  netmask: string
  mac: string
  isConnected?: boolean
}

/**
 * Network status information
 */
export interface NetworkStatus {
  isConnected: boolean
  primaryInterface: NetworkInterface | null
  interfaces: NetworkInterface[]
}
