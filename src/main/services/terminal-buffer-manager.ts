/**
 * Terminal Buffer Manager for Main Process
 * Manages terminal output buffers to prevent data loss during re-renders
 */
export class TerminalBufferManager {
  private static instance: TerminalBufferManager
  private readonly buffers: Map<string, string[]> = new Map()
  private readonly MAX_BUFFER_LINES = 2000

  /**
   * Get singleton instance
   */
  static getInstance(): TerminalBufferManager {
    if (!TerminalBufferManager.instance) {
      TerminalBufferManager.instance = new TerminalBufferManager()
    }
    return TerminalBufferManager.instance
  }

  /**
   * Save data to terminal buffer
   * @param terminalId - Terminal identifier
   * @param data - Terminal output data
   */
  saveToBuffer(terminalId: string, data: string): void {
    try {
      if (!data || typeof data !== 'string') {
        return
      }

      // Initialize buffer if not exists
      if (!this.buffers.has(terminalId)) {
        this.buffers.set(terminalId, [])
      }

      const buffer = this.buffers.get(terminalId)!

      // Split data by lines and add to buffer
      const lines = data.split('\n')

      // If the data doesn't end with newline, the last element should be merged with existing last line
      if (buffer.length > 0 && !data.startsWith('\n') && lines.length > 0) {
        buffer[buffer.length - 1] += lines[0]
        lines.shift() // Remove first element as it's already merged
      }

      // Add remaining lines
      buffer.push(...lines)

      // Trim buffer to prevent memory overflow
      this.trimBuffer(buffer)
    } catch (error) {
      console.error(`Failed to save buffer for terminal ${terminalId}:`, error)
    }
  }

  /**
   * Get buffer for specific terminal
   * @param terminalId - Terminal identifier
   * @returns Buffer lines array
   */
  getBuffer(terminalId: string): string[] {
    const buffer = this.buffers.get(terminalId)
    return buffer ? [...buffer] : [] // Return copy to prevent external modification
  }

  /**
   * Get buffer as string for specific terminal
   * @param terminalId - Terminal identifier
   * @returns Buffer as joined string
   */
  getBufferAsString(terminalId: string): string {
    const buffer = this.getBuffer(terminalId)
    return buffer.join('\n')
  }

  /**
   * Clear buffer for specific terminal
   * @param terminalId - Terminal identifier
   */
  clearBuffer(terminalId: string): void {
    this.buffers.delete(terminalId)
  }

  /**
   * Get all buffers (for debugging or bulk operations)
   * @returns Record of all terminal buffers
   */
  getAllBuffers(): Record<string, string[]> {
    const result: Record<string, string[]> = {}
    for (const [terminalId, buffer] of this.buffers.entries()) {
      result[terminalId] = [...buffer] // Return copies
    }
    return result
  }

  /**
   * Get buffer statistics
   * @returns Buffer statistics
   */
  getStats(): { totalTerminals: number; totalLines: number; memoryUsage: number } {
    let totalLines = 0
    let memoryUsage = 0

    for (const [terminalId, buffer] of this.buffers.entries()) {
      totalLines += buffer.length
      memoryUsage += buffer.reduce((acc, line) => acc + line.length, 0)
      memoryUsage += terminalId.length
    }

    return {
      totalTerminals: this.buffers.size,
      totalLines,
      memoryUsage: memoryUsage * 2 // Rough estimate (UTF-16)
    }
  }

  /**
   * Check if terminal has buffer
   * @param terminalId - Terminal identifier
   * @returns Whether buffer exists
   */
  hasBuffer(terminalId: string): boolean {
    return this.buffers.has(terminalId)
  }

  /**
   * Trim buffer to prevent memory overflow
   * @param buffer - Buffer array to trim
   */
  private trimBuffer(buffer: string[]): void {
    if (buffer.length > this.MAX_BUFFER_LINES) {
      const trimCount = buffer.length - this.MAX_BUFFER_LINES
      buffer.splice(0, trimCount)
    }
  }

  /**
   * Cleanup buffers when terminal manager shuts down
   */
  cleanup(): void {
    this.buffers.clear()
  }

  /**
   * Cleanup old buffers to prevent memory leaks
   * Should be called periodically if terminals are frequently created/destroyed
   */
  cleanupOrphanedBuffers(activeTerminalIds: string[]): void {
    const activeSet = new Set(activeTerminalIds)
    const orphanedIds: string[] = []

    for (const terminalId of this.buffers.keys()) {
      if (!activeSet.has(terminalId)) {
        orphanedIds.push(terminalId)
      }
    }

    for (const orphanedId of orphanedIds) {
      this.clearBuffer(orphanedId)
    }
  }
}
