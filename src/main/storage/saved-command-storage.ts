import { BaseStorage } from './base-storage'
import type { SavedCommand } from '../types/ssh'

export class SavedCommandStorage extends BaseStorage {
  constructor() {
    super('saved-commands.json')
  }

  /**
   * Generate a unique ID
   */
  private generateId(): string {
    return Date.now().toString(36) + Math.random().toString(36).substring(2)
  }

  /**
   * Get all saved commands
   */
  async getAll(): Promise<SavedCommand[]> {
    return await this.readData<SavedCommand>()
  }

  /**
   * Get a saved command by ID
   */
  async get(id: string): Promise<SavedCommand | null> {
    const commands = await this.getAll()
    return commands.find((command) => command.id === id) || null
  }

  /**
   * Save a command
   */
  async save(command: SavedCommand): Promise<void> {
    const commands = await this.getAll()
    const existingIndex = commands.findIndex((c) => c.id === command.id)

    if (existingIndex >= 0) {
      commands[existingIndex] = command
    } else {
      commands.push(command)
    }

    await this.writeData(commands)
  }

  /**
   * Delete a command
   */
  async delete(id: string): Promise<void> {
    const commands = await this.getAll()
    const filteredCommands = commands.filter((command) => command.id !== id)
    await this.writeData(filteredCommands)
  }

  /**
   * Create a new saved command
   */
  async create(data: Omit<SavedCommand, 'id' | 'created' | 'updated'>): Promise<SavedCommand> {
    const command: SavedCommand = {
      ...data,
      id: this.generateId(),
      created: new Date(),
      updated: new Date()
    }

    await this.save(command)
    return command
  }

  /**
   * Update an existing saved command
   */
  async update(
    id: string,
    data: Partial<Omit<SavedCommand, 'id' | 'created'>>
  ): Promise<SavedCommand | null> {
    const command = await this.get(id)
    if (!command) return null

    const updatedCommand: SavedCommand = {
      ...command,
      ...data,
      updated: new Date()
    }

    await this.save(updatedCommand)
    return updatedCommand
  }
}
