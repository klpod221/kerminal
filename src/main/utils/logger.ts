/**
 * Simple console logger implementation
 */
import { ILogger } from '../interfaces/application.interface'

export class ConsoleLogger implements ILogger {
  constructor(private readonly prefix: string = 'KerMinal') {}

  info(message: string, ...args: unknown[]): void {
    console.log(`[${this.prefix}] INFO: ${message}`, ...args)
  }

  warn(message: string, ...args: unknown[]): void {
    console.warn(`[${this.prefix}] WARN: ${message}`, ...args)
  }

  error(message: string, error?: Error, ...args: unknown[]): void {
    console.error(`[${this.prefix}] ERROR: ${message}`, error, ...args)
  }

  debug(message: string, ...args: unknown[]): void {
    console.debug(`[${this.prefix}] DEBUG: ${message}`, ...args)
  }
}
