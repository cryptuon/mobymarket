import type { WebSocketMessage } from '@/types'

export type ConnectionState = 'connecting' | 'connected' | 'disconnected' | 'error'

export interface WebSocketOptions {
  url: string
  protocols?: string[]
  autoReconnect?: boolean
  reconnectInterval?: number
  maxReconnectAttempts?: number
  heartbeatInterval?: number
  timeout?: number
}

export interface WebSocketSubscription {
  channel: string
  handler: (data: any) => void
  id: string
}

export class WebSocketService {
  private ws: WebSocket | null = null
  private options: Required<WebSocketOptions>
  private state: ConnectionState = 'disconnected'
  private subscriptions = new Map<string, WebSocketSubscription>()
  private reconnectAttempts = 0
  private reconnectTimer: NodeJS.Timeout | null = null
  private heartbeatTimer: NodeJS.Timeout | null = null
  private messageQueue: any[] = []

  // Event handlers
  private onStateChange: ((state: ConnectionState) => void) | null = null
  private onError: ((error: Event) => void) | null = null
  private onMessage: ((message: WebSocketMessage) => void) | null = null

  constructor(options: WebSocketOptions) {
    this.options = {
      autoReconnect: true,
      reconnectInterval: 5000,
      maxReconnectAttempts: 5,
      heartbeatInterval: 30000,
      timeout: 10000,
      protocols: [],
      ...options
    }
  }

  // Connection management
  async connect(): Promise<void> {
    if (this.state === 'connecting' || this.state === 'connected') {
      return
    }

    this.setState('connecting')

    try {
      this.ws = new WebSocket(this.options.url, this.options.protocols)
      this.setupEventListeners()

      // Wait for connection with timeout
      await this.waitForConnection()

      this.setState('connected')
      this.reconnectAttempts = 0
      this.startHeartbeat()
      this.processMessageQueue()

    } catch (error) {
      this.setState('error')
      if (this.options.autoReconnect && this.reconnectAttempts < this.options.maxReconnectAttempts) {
        this.scheduleReconnect()
      }
      throw error
    }
  }

  disconnect(): void {
    this.setState('disconnected')
    this.stopReconnect()
    this.stopHeartbeat()

    if (this.ws) {
      this.ws.close(1000, 'Client disconnecting')
      this.ws = null
    }
  }

  // Message handling
  send(message: any): boolean {
    if (this.state !== 'connected' || !this.ws) {
      // Queue message for later if auto-reconnect is enabled
      if (this.options.autoReconnect) {
        this.messageQueue.push(message)
      }
      return false
    }

    try {
      const serialized = typeof message === 'string' ? message : JSON.stringify(message)
      this.ws.send(serialized)
      return true
    } catch (error) {
      console.error('Failed to send WebSocket message:', error)
      return false
    }
  }

  // Subscription management
  subscribe(channel: string, handler: (data: any) => void): string {
    const id = `sub-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`

    this.subscriptions.set(id, {
      channel,
      handler,
      id
    })

    // Send subscription message if connected
    this.send({
      type: 'subscribe',
      channel,
      id
    })

    return id
  }

  unsubscribe(subscriptionId: string): void {
    const subscription = this.subscriptions.get(subscriptionId)
    if (!subscription) return

    this.subscriptions.delete(subscriptionId)

    // Send unsubscribe message if connected
    this.send({
      type: 'unsubscribe',
      channel: subscription.channel,
      id: subscriptionId
    })
  }

  unsubscribeFromChannel(channel: string): void {
    const toRemove: string[] = []

    for (const [id, sub] of this.subscriptions) {
      if (sub.channel === channel) {
        toRemove.push(id)
      }
    }

    toRemove.forEach(id => this.unsubscribe(id))
  }

  // State management
  getState(): ConnectionState {
    return this.state
  }

  isConnected(): boolean {
    return this.state === 'connected'
  }

  // Event listeners
  onStateChanged(handler: (state: ConnectionState) => void): void {
    this.onStateChange = handler
  }

  onErrorOccurred(handler: (error: Event) => void): void {
    this.onError = handler
  }

  onMessageReceived(handler: (message: WebSocketMessage) => void): void {
    this.onMessage = handler
  }

  // Private methods
  private setState(newState: ConnectionState): void {
    if (this.state !== newState) {
      this.state = newState
      this.onStateChange?.(newState)
    }
  }

  private setupEventListeners(): void {
    if (!this.ws) return

    this.ws.onopen = () => {
      console.log('WebSocket connected')
    }

    this.ws.onclose = (event) => {
      console.log('WebSocket disconnected:', event.code, event.reason)

      if (this.state !== 'disconnected') {
        this.setState('disconnected')

        if (this.options.autoReconnect &&
            this.reconnectAttempts < this.options.maxReconnectAttempts &&
            event.code !== 1000) { // Don't reconnect on normal closure
          this.scheduleReconnect()
        }
      }
    }

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error)
      this.onError?.(error)
      this.setState('error')
    }

    this.ws.onmessage = (event) => {
      try {
        const message: WebSocketMessage = JSON.parse(event.data)
        this.handleMessage(message)
        this.onMessage?.(message)
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error)
      }
    }
  }

  private handleMessage(message: WebSocketMessage): void {
    // Handle heartbeat/pong messages
    if (message.type === 'pong') {
      return
    }

    // Route message to appropriate subscription handlers
    for (const subscription of this.subscriptions.values()) {
      if (message.type === subscription.channel ||
          (message as any).channel === subscription.channel) {
        try {
          subscription.handler(message.data || message)
        } catch (error) {
          console.error(`Error in subscription handler for ${subscription.channel}:`, error)
        }
      }
    }
  }

  private async waitForConnection(): Promise<void> {
    return new Promise((resolve, reject) => {
      if (!this.ws) {
        reject(new Error('WebSocket not initialized'))
        return
      }

      const timeout = setTimeout(() => {
        reject(new Error('WebSocket connection timeout'))
      }, this.options.timeout)

      const onOpen = () => {
        clearTimeout(timeout)
        resolve()
      }

      const onError = (error: Event) => {
        clearTimeout(timeout)
        reject(error)
      }

      this.ws.addEventListener('open', onOpen, { once: true })
      this.ws.addEventListener('error', onError, { once: true })
    })
  }

  private scheduleReconnect(): void {
    this.reconnectAttempts++

    const delay = Math.min(
      this.options.reconnectInterval * Math.pow(2, this.reconnectAttempts - 1),
      30000 // Max 30 seconds
    )

    console.log(`Scheduling reconnect attempt ${this.reconnectAttempts} in ${delay}ms`)

    this.reconnectTimer = setTimeout(() => {
      if (this.state !== 'connected') {
        this.connect().catch(console.error)
      }
    }, delay)
  }

  private stopReconnect(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }
    this.reconnectAttempts = 0
  }

  private startHeartbeat(): void {
    this.stopHeartbeat()

    this.heartbeatTimer = setInterval(() => {
      if (this.state === 'connected') {
        this.send({ type: 'ping', timestamp: Date.now() })
      }
    }, this.options.heartbeatInterval)
  }

  private stopHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer)
      this.heartbeatTimer = null
    }
  }

  private processMessageQueue(): void {
    while (this.messageQueue.length > 0) {
      const message = this.messageQueue.shift()
      this.send(message)
    }
  }

  // Cleanup
  destroy(): void {
    this.disconnect()
    this.subscriptions.clear()
    this.messageQueue = []
    this.onStateChange = null
    this.onError = null
    this.onMessage = null
  }
}

// Singleton instance for global use
export const globalWebSocket = new WebSocketService({
  url: process.env.VITE_WS_URL || 'wss://api.mobymarket.com/ws',
  autoReconnect: true,
  reconnectInterval: 3000,
  maxReconnectAttempts: 10,
  heartbeatInterval: 30000
})