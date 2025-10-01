import { invoke } from '@tauri-apps/api/core'

export interface AuthEvent {
  type: 'SessionUnlocked' | 'SessionLocked' | 'AutoUnlockAttempted'
  timestamp: string
  // SessionUnlocked
  via_auto_unlock?: boolean
  // SessionLocked
  reason?: {
    Manual?: null
    Timeout?: null
    Error?: string
  }
  // AutoUnlockAttempted
  success?: boolean
  error?: string | null
}

export interface AuthSession {
  isSetup: boolean
  isUnlocked: boolean
  autoUnlockEnabled: boolean
  keychainAvailable: boolean
  sessionActive: boolean
  sessionExpiresAt?: string
  loadedDeviceCount: number
  timestamp: string
}

export interface AuthEventsResponse {
  events: AuthEvent[]
  timestamp: string
}

/**
 * Service for handling auth events and session management
 */
class AuthEventsService {
  private pollingInterval: number | null = null
  private statusListeners: Set<(status: AuthSession) => void> = new Set()

  /**
   * Start polling for auth events
   */
  async startPolling(intervalMs: number = 2000): Promise<void> {
    if (this.pollingInterval) {
      this.stopPolling()
    }

    this.pollingInterval = window.setInterval(async () => {
      try {
        // Get latest auth session status
        const status = await this.getAuthSessionStatus()

        // Notify status listeners
        this.statusListeners.forEach(listener => listener(status))

        // Note: For now we're not implementing event polling since we don't store events in backend
        // The status polling above is sufficient for most use cases
      } catch (error) {
        console.error('Failed to poll auth status:', error)
      }
    }, intervalMs)
  }

  /**
   * Stop polling for auth events
   */
  stopPolling(): void {
    if (this.pollingInterval) {
      clearInterval(this.pollingInterval)
      this.pollingInterval = null
    }
  }

  /**
   * Get current auth session status
   */
  async getAuthSessionStatus(): Promise<AuthSession> {
    return await invoke<AuthSession>('get_auth_session_status')
  }

  /**
   * Notify backend that session was unlocked
   */
  async notifySessionUnlocked(): Promise<void> {
    await invoke('notify_session_unlocked')
  }

  /**
   * Notify backend that session was locked
   */
  async notifySessionLocked(reason: string = 'manual'): Promise<void> {
    await invoke('notify_session_locked', { reason })
  }

  /**
   * Subscribe to auth events (currently uses polling)
   */
  async subscribeAuthEvents(): Promise<void> {
    await invoke('subscribe_auth_events')
  }

  /**
   * Get auth events since timestamp
   */
  async getAuthEvents(since?: string): Promise<AuthEventsResponse> {
    return await invoke<AuthEventsResponse>('get_auth_events', { since })
  }



  /**
   * Add status change listener
   */
  addStatusListener(listener: (status: AuthSession) => void): void {
    this.statusListeners.add(listener)
  }

  /**
   * Remove status change listener
   */
  removeStatusListener(listener: (status: AuthSession) => void): void {
    this.statusListeners.delete(listener)
  }

  /**
   * Cleanup - stop polling and clear listeners
   */
  cleanup(): void {
    this.stopPolling()
    this.statusListeners.clear()
  }
}

// Export singleton instance
export const authEventsService = new AuthEventsService()
