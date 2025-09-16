import { createApp, App as VueApp } from 'vue'
import Message from '../components/ui/Message.vue'

export interface MessageOptions {
  type?: 'success' | 'error' | 'warning' | 'info' | 'loading'
  title?: string
  content: string
  duration?: number
  closable?: boolean
}

class MessageService {
  private readonly instances: Set<VueApp> = new Set()

  private create(options: MessageOptions): Promise<void> {
    return new Promise((resolve) => {
      const app = createApp(Message, {
        ...options,
        onClose: () => {
          this.instances.delete(app)
          app.unmount()
          resolve()
        }
      })

      this.instances.add(app)
      const container = document.createElement('div')
      document.body.appendChild(container)
      app.mount(container)
    })
  }

  success(content: string, title?: string, duration = 3000): Promise<void> {
    return this.create({
      type: 'success',
      content,
      title,
      duration
    })
  }

  error(content: string, title?: string, duration = 4000): Promise<void> {
    return this.create({
      type: 'error',
      content,
      title,
      duration
    })
  }

  warning(content: string, title?: string, duration = 3000): Promise<void> {
    return this.create({
      type: 'warning',
      content,
      title,
      duration
    })
  }

  info(content: string, title?: string, duration = 3000): Promise<void> {
    return this.create({
      type: 'info',
      content,
      title,
      duration
    })
  }

  loading(content: string, title?: string): Promise<void> {
    return this.create({
      type: 'loading',
      content,
      title,
      duration: 0,
      closable: false
    })
  }

  destroy(): void {
    this.instances.forEach((app) => {
      app.unmount()
    })
    this.instances.clear()
  }
}

export const message = new MessageService()
