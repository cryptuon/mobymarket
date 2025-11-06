/**
 * API Services Index
 * Central export point for all API services
 */

// Base API client and utilities
export {
  apiClient,
  ApiClient,
  isApiError,
  handleApiError,
  CancelableRequest,
  RequestQueue,
  requestQueue,
  RateLimiter,
  rateLimiter
} from './base'

// Authentication service
export { authService, AuthService } from './auth'
export type {
  LoginCredentials,
  RegisterData,
  User,
  UserPreferences,
  UserSubscription,
  AuthTokens,
  AuthResponse,
  ResetPasswordData,
  ConfirmResetPasswordData,
  ChangePasswordData,
  TwoFactorSetupData,
  TwoFactorVerifyData,
  SessionInfo
} from './auth'

// Portfolio service
export { portfolioService, PortfolioService } from './portfolio'
export type {
  Portfolio,
  Position,
  Transaction,
  PortfolioSettings,
  PortfolioSummary,
  PerformanceMetrics,
  AllocationData,
  RiskMetrics,
  RebalanceRecommendation,
  PortfolioComparison
} from './portfolio'

// Market data service
export { marketService, MarketService } from './market'
export type {
  Asset,
  PriceData,
  ChartData,
  MarketStats,
  TrendingAsset,
  WhaleTransaction,
  MarketSentiment,
  NewsArticle,
  SocialMetrics,
  DeFiProtocol,
  LiquidityPool,
  GasTracker,
  ExchangeInfo
} from './market'

// Trading service
export { tradingService, TradingService } from './trading'
export type {
  TradingAccount,
  Order,
  OrderFill,
  Position as TradingPosition,
  TradingPair,
  OrderBook,
  Trade,
  Ticker,
  Kline,
  TradingStrategy,
  TradingSignal,
  RiskAssessment
} from './trading'

// Common types
export type {
  ApiResponse,
  ApiError,
  PaginatedResponse,
  RequestConfig
} from './base'

// API Health check utility
export const checkApiHealth = async (): Promise<boolean> => {
  try {
    return await apiClient.healthCheck()
  } catch (error) {
    console.error('API health check failed:', error)
    return false
  }
}

// API Configuration utilities
export const updateApiConfig = (config: {
  baseURL?: string
  timeout?: number
  maxRetries?: number
}) => {
  if (config.baseURL) {
    apiClient.updateBaseURL(config.baseURL)
  }

  apiClient.updateConfig({
    timeout: config.timeout,
    maxRetries: config.maxRetries
  })
}

export const getApiConfig = () => {
  return apiClient.getConfig()
}

// Error handling utilities
export const createApiErrorHandler = (defaultMessage: string = 'An error occurred') => {
  return (error: any) => {
    const message = handleApiError(error, defaultMessage)
    console.error('API Error:', message, error)
    return message
  }
}

// Request cancellation helper
export const createCancelableApiRequest = <T>(
  requestFn: (signal: AbortSignal) => Promise<T>
): CancelableRequest<T> => {
  return new CancelableRequest(requestFn)
}

// Batch request helper
export const batchApiRequests = async <T>(
  requests: (() => Promise<T>)[],
  maxConcurrent: number = 5
): Promise<T[]> => {
  const queue = new RequestQueue(maxConcurrent)
  return Promise.all(requests.map(req => queue.add(req)))
}

// Rate limiting helper
export const withRateLimit = async <T>(
  requestFn: () => Promise<T>,
  limiter: RateLimiter = rateLimiter
): Promise<T> => {
  const canProceed = await limiter.checkLimit()

  if (!canProceed) {
    const resetTime = limiter.getResetTime()
    const waitTime = resetTime - Date.now()

    if (waitTime > 0) {
      throw new Error(`Rate limit exceeded. Try again in ${Math.ceil(waitTime / 1000)} seconds.`)
    }
  }

  return requestFn()
}

// Retry mechanism for failed requests
export const retryApiRequest = async <T>(
  requestFn: () => Promise<T>,
  options: {
    maxRetries?: number
    backoffMs?: number
    retryCondition?: (error: any) => boolean
  } = {}
): Promise<T> => {
  const {
    maxRetries = 3,
    backoffMs = 1000,
    retryCondition = (error) => !error.response || error.response.status >= 500
  } = options

  let lastError: any

  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return await requestFn()
    } catch (error) {
      lastError = error

      if (attempt === maxRetries || !retryCondition(error)) {
        throw error
      }

      // Exponential backoff
      const delay = backoffMs * Math.pow(2, attempt)
      await new Promise(resolve => setTimeout(resolve, delay))
    }
  }

  throw lastError
}

// Pagination helper
export const getAllPages = async <T>(
  requestFn: (page: number, limit: number) => Promise<ApiResponse<PaginatedResponse<T>>>,
  limit: number = 50
): Promise<T[]> => {
  const allItems: T[] = []
  let page = 1
  let hasMore = true

  while (hasMore) {
    const response = await requestFn(page, limit)
    const { data: items, pagination } = response.data

    allItems.push(...items)
    hasMore = pagination.hasNext
    page++
  }

  return allItems
}

// WebSocket connection manager
export class WebSocketManager {
  private connections: Map<string, WebSocket> = new Map()

  connect(key: string, url: string, protocols?: string | string[]): WebSocket {
    // Close existing connection if any
    this.disconnect(key)

    const ws = new WebSocket(url, protocols)
    this.connections.set(key, ws)

    ws.onclose = () => {
      this.connections.delete(key)
    }

    return ws
  }

  disconnect(key: string): void {
    const ws = this.connections.get(key)
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.close()
    }
    this.connections.delete(key)
  }

  disconnectAll(): void {
    for (const [key] of this.connections) {
      this.disconnect(key)
    }
  }

  getConnection(key: string): WebSocket | undefined {
    return this.connections.get(key)
  }

  isConnected(key: string): boolean {
    const ws = this.connections.get(key)
    return ws?.readyState === WebSocket.OPEN
  }

  getActiveConnections(): string[] {
    return Array.from(this.connections.keys()).filter(key => this.isConnected(key))
  }
}

// Export default WebSocket manager instance
export const webSocketManager = new WebSocketManager()

// API service initialization
export const initializeApiServices = async (config?: {
  baseURL?: string
  timeout?: number
  maxRetries?: number
  enableHealthCheck?: boolean
}): Promise<boolean> => {
  try {
    // Update configuration if provided
    if (config) {
      updateApiConfig(config)
    }

    // Perform health check if enabled
    if (config?.enableHealthCheck !== false) {
      const isHealthy = await checkApiHealth()
      if (!isHealthy) {
        console.warn('API health check failed, but continuing with initialization')
      }
    }

    console.log('API services initialized successfully')
    return true
  } catch (error) {
    console.error('Failed to initialize API services:', error)
    return false
  }
}

// Cleanup function for when the app is destroyed
export const cleanupApiServices = (): void => {
  // Close all WebSocket connections
  webSocketManager.disconnectAll()

  // Clear request queue
  requestQueue.clear()

  console.log('API services cleaned up')
}

// Development utilities
export const getDebugInfo = () => {
  if (import.meta.env.DEV) {
    return {
      config: getApiConfig(),
      activeConnections: webSocketManager.getActiveConnections(),
      queueSize: requestQueue.queueSize,
      activeRequests: requestQueue.activeRequests,
      rateLimitRemaining: rateLimiter.getRemainingRequests(),
      rateLimitReset: new Date(rateLimiter.getResetTime())
    }
  }
  return null
}

// Export everything for convenience
export default {
  // Services
  auth: authService,
  portfolio: portfolioService,
  market: marketService,
  trading: tradingService,

  // Utilities
  client: apiClient,
  webSocket: webSocketManager,
  queue: requestQueue,
  rateLimit: rateLimiter,

  // Helper functions
  checkHealth: checkApiHealth,
  updateConfig: updateApiConfig,
  getConfig: getApiConfig,
  initialize: initializeApiServices,
  cleanup: cleanupApiServices,
  debug: getDebugInfo
}