/**
 * Terminal Buffer Manager for Renderer Process
 * Manages terminal output buffers and synchronizes with main process
 */
export class TerminalBufferManager {
  private static instance: TerminalBufferManager
  private readonly localBuffers: Map<string, string[]> = new Map()
  private readonly MAX_LOCAL_BUFFER_LINES = 2000

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
   * Save data to local buffer
   * @param terminalId - Terminal identifier
   * @param data - Terminal output data
   */
  saveToLocalBuffer(terminalId: string, data: string): void {
    try {
      if (!data || typeof data !== 'string') {
        return
      }

      // Initialize buffer if not exists
      if (!this.localBuffers.has(terminalId)) {
        this.localBuffers.set(terminalId, [])
      }

      const buffer = this.localBuffers.get(terminalId)!

      // Split data by lines and add to buffer
      const lines = data.split('\n')

      // If the data doesn't end with newline, merge with existing last line
      if (buffer.length > 0 && !data.startsWith('\n') && lines.length > 0) {
        buffer[buffer.length - 1] += lines[0]
        lines.shift()
      }

      // Add remaining lines
      buffer.push(...lines)

      // Trim buffer to prevent memory overflow
      this.trimLocalBuffer(buffer)
    } catch (error) {
      console.error(`Failed to save to local buffer for terminal ${terminalId}:`, error)
    }
  }

  /**
   * Get local buffer for specific terminal
   * @param terminalId - Terminal identifier
   * @returns Buffer lines array
   */
  getLocalBuffer(terminalId: string): string[] {
    const buffer = this.localBuffers.get(terminalId)
    return buffer ? [...buffer] : []
  }

  /**
   * Get buffer from main process
   * @param terminalId - Terminal identifier
   * @returns Promise of buffer lines array
   */
  async getBufferFromMain(terminalId: string): Promise<string[]> {
    try {
      const buffer = await window.api.invoke('terminal.buffer.get', terminalId)
      return Array.isArray(buffer) ? buffer : []
    } catch (error) {
      console.error(`Failed to get buffer from main for terminal ${terminalId}:`, error)
      return []
    }
  }

  /**
   * Get buffer as string from main process
   * @param terminalId - Terminal identifier
   * @returns Promise of buffer string
   */
  async getBufferStringFromMain(terminalId: string): Promise<string> {
    try {
      const bufferString = await window.api.invoke('terminal.buffer.getString', terminalId)
      return typeof bufferString === 'string' ? bufferString : ''
    } catch (error) {
      console.error(`Failed to get buffer string from main for terminal ${terminalId}:`, error)
      return ''
    }
  }

  /**
   * Check if terminal has buffer in main process
   * @param terminalId - Terminal identifier
   * @returns Promise of boolean
   */
  async hasBufferInMain(terminalId: string): Promise<boolean> {
    try {
      const hasBuffer = await window.api.invoke('terminal.buffer.has', terminalId)
      return Boolean(hasBuffer)
    } catch (error) {
      console.error(`Failed to check buffer in main for terminal ${terminalId}:`, error)
      return false
    }
  }

  /**
   * Get buffer statistics from main process
   * @returns Promise of buffer statistics
   */
  async getBufferStats(): Promise<{
    totalTerminals: number
    totalLines: number
    memoryUsage: number
  }> {
    try {
      const stats = await window.api.invoke('terminal.buffer.stats')
      return stats &&
        typeof stats === 'object' &&
        'totalTerminals' in stats &&
        'totalLines' in stats &&
        'memoryUsage' in stats
        ? (stats as { totalTerminals: number; totalLines: number; memoryUsage: number })
        : { totalTerminals: 0, totalLines: 0, memoryUsage: 0 }
    } catch (error) {
      console.error('Failed to get buffer stats from main:', error)
      return { totalTerminals: 0, totalLines: 0, memoryUsage: 0 }
    }
  }

  /**
   * Clear local buffer for specific terminal
   * @param terminalId - Terminal identifier
   */
  clearLocalBuffer(terminalId: string): void {
    this.localBuffers.delete(terminalId)
  }

  /**
   * Restore terminal buffer from main process
   * @param terminalId - Terminal identifier
   * @param terminal - xterm.js Terminal instance
   * @returns Promise of success boolean
   */
  async restoreBuffer(
    terminalId: string,
    terminal: { clear: () => void; write: (data: string) => void }
  ): Promise<boolean> {
    try {
      // Check if buffer exists in main process
      const hasBuffer = await this.hasBufferInMain(terminalId)
      if (!hasBuffer) {
        return false
      }

      // Get buffer from main process
      const bufferString = await this.getBufferStringFromMain(terminalId)
      if (!bufferString) {
        return false
      }

      // Clear terminal and write buffer
      terminal.clear()
      terminal.write(bufferString)

      return true
    } catch (error) {
      console.error(`Failed to restore buffer for terminal ${terminalId}:`, error)
      return false
    }
  }

  /**
   * Trigger cleanup of orphaned buffers in main process
   */
  async triggerCleanup(): Promise<void> {
    try {
      window.api.send('terminal.buffer.cleanup', {})
    } catch (error) {
      console.error('Failed to trigger buffer cleanup:', error)
    }
  }

  /**
   * Get combined buffer stats (local + main)
   * @returns Promise of combined statistics
   */
  async getCombinedStats(): Promise<{
    local: { terminals: number; lines: number }
    main: { totalTerminals: number; totalLines: number; memoryUsage: number }
  }> {
    try {
      const mainStats = await this.getBufferStats()

      let localLines = 0
      for (const buffer of this.localBuffers.values()) {
        localLines += buffer.length
      }

      return {
        local: {
          terminals: this.localBuffers.size,
          lines: localLines
        },
        main: mainStats
      }
    } catch (error) {
      console.error('Failed to get combined stats:', error)
      return {
        local: { terminals: 0, lines: 0 },
        main: { totalTerminals: 0, totalLines: 0, memoryUsage: 0 }
      }
    }
  }

  /**
   * Cleanup all local buffers
   */
  cleanup(): void {
    this.localBuffers.clear()
  }

  /**
   * Trim local buffer to prevent memory overflow
   * @param buffer - Buffer array to trim
   */
  private trimLocalBuffer(buffer: string[]): void {
    if (buffer.length > this.MAX_LOCAL_BUFFER_LINES) {
      const trimCount = buffer.length - this.MAX_LOCAL_BUFFER_LINES
      buffer.splice(0, trimCount)
    }
  }
}
