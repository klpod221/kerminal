import { BaseService } from '../base/base-service'
import { SavedCommandStorage } from '../storage/saved-command-storage'
import type { SavedCommand, CommandExecutionOptions } from '../types/ssh'
import type { ILogger } from '../interfaces/application.interface'

export class SavedCommandService extends BaseService {
  private readonly storage: SavedCommandStorage

  constructor(logger?: ILogger) {
    super('SavedCommandService', logger)
    this.storage = new SavedCommandStorage()
  }

  protected async onInitialize(): Promise<void> {
    // Initialize storage if needed
    this.logger?.info('SavedCommandService initialized')
  }

  protected onCleanup(): void {
    // Cleanup if needed
    this.logger?.info('SavedCommandService cleaned up')
  }

  /**
   * Get all saved commands
   */
  async getAllCommands(): Promise<SavedCommand[]> {
    try {
      return await this.storage.getAll()
    } catch (error) {
      this.logger?.error('Failed to get all commands:', error as Error)
      throw error
    }
  }

  /**
   * Create a new saved command
   */
  async createCommand(
    data: Omit<SavedCommand, 'id' | 'created' | 'updated'>
  ): Promise<SavedCommand> {
    try {
      this.logger?.info('Creating new saved command:', data.name)
      return await this.storage.create(data)
    } catch (error) {
      this.logger?.error('Failed to create command:', error as Error)
      throw error
    }
  }

  /**
   * Update an existing saved command
   */
  async updateCommand(
    id: string,
    data: Partial<Omit<SavedCommand, 'id' | 'created'>>
  ): Promise<SavedCommand | null> {
    try {
      this.logger?.info('Updating saved command:', id)
      return await this.storage.update(id, data)
    } catch (error) {
      this.logger?.error('Failed to update command:', error as Error)
      throw error
    }
  }

  /**
   * Delete a saved command
   */
  async deleteCommand(id: string): Promise<void> {
    try {
      this.logger?.info('Deleting saved command:', id)
      await this.storage.delete(id)
    } catch (error) {
      this.logger?.error('Failed to delete command:', error as Error)
      throw error
    }
  }

  /**
   * Execute a command in a terminal
   */
  async executeCommand(options: CommandExecutionOptions): Promise<void> {
    try {
      this.logger?.info('Executing command in terminal:', options.terminalId)

      // Send the command to the terminal via IPC
      const { ipcMain } = await import('electron')
      ipcMain.emit('terminal-send-command', null, {
        terminalId: options.terminalId,
        command: options.command,
        addToHistory: options.addToHistory !== false
      })
    } catch (error) {
      this.logger?.error('Failed to execute command:', error as Error)
      throw error
    }
  }

  /**
   * Copy command to clipboard
   */
  async copyCommandToClipboard(command: string): Promise<void> {
    try {
      const { clipboard } = await import('electron')
      clipboard.writeText(command)
      this.logger?.info('Command copied to clipboard')
    } catch (error) {
      this.logger?.error('Failed to copy command to clipboard:', error as Error)
      throw error
    }
  }
}
