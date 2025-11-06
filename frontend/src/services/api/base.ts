import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios'
import { useAuthStore } from '@stores/auth'

/**
 * Base API configuration and utilities
 */

export interface ApiResponse<T = any> {
  data: T
  message?: string
  success: boolean
  timestamp: string
}

export interface ApiError {
  message: string
  code: string
  details?: any
  timestamp: string
}

export interface PaginatedResponse<T> {
  data: T[]
  pagination: {
    page: number
    limit: number
    total: number
    totalPages: number
    hasNext: boolean
    hasPrev: boolean
  }
}

export interface RequestConfig extends AxiosRequestConfig {
  skipAuth?: boolean
  timeout?: number
  retries?: number
}

class ApiClient {
  private instance: AxiosInstance
  private baseURL: string
  private defaultTimeout: number = 30000
  private maxRetries: number = 3

  constructor(baseURL: string = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000/api') {
    this.baseURL = baseURL
    this.instance = this.createAxiosInstance()
    this.setupInterceptors()
  }

  private createAxiosInstance(): AxiosInstance {
    return axios.create({
      baseURL: this.baseURL,
      timeout: this.defaultTimeout,
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json'
      }
    })
  }

  private setupInterceptors(): void {
    // Request interceptor
    this.instance.interceptors.request.use(
      (config) => {
        // Add auth token if available and not skipped
        if (!config.skipAuth) {
          const authStore = useAuthStore()
          const token = authStore.token

          if (token) {
            config.headers.Authorization = `Bearer ${token}`
          }
        }

        // Add request timestamp
        config.metadata = {
          startTime: Date.now()
        }

        // Log request in development
        if (import.meta.env.DEV) {
          console.log(`🚀 API Request: ${config.method?.toUpperCase()} ${config.url}`, {
            params: config.params,
            data: config.data
          })
        }

        return config
      },
      (error) => {
        console.error('❌ Request Error:', error)
        return Promise.reject(error)
      }
    )

    // Response interceptor
    this.instance.interceptors.response.use(
      (response) => {
        // Calculate request duration
        const duration = Date.now() - response.config.metadata?.startTime || 0

        // Log response in development
        if (import.meta.env.DEV) {
          console.log(`✅ API Response: ${response.config.method?.toUpperCase()} ${response.config.url} (${duration}ms)`, {
            status: response.status,
            data: response.data
          })
        }

        return response
      },
      async (error) => {
        const config = error.config
        const status = error.response?.status

        // Log error
        console.error('❌ API Error:', {
          url: config?.url,
          method: config?.method?.toUpperCase(),
          status,
          message: error.message,
          data: error.response?.data
        })

        // Handle token expiration
        if (status === 401 && !config.skipAuth) {
          const authStore = useAuthStore()
          await authStore.logout()
          // Redirect to login page
          window.location.href = '/auth/login'
          return Promise.reject(error)
        }

        // Retry logic for network errors
        if (this.shouldRetry(error) && config.retries !== undefined && config.retries > 0) {
          config.retries--

          // Exponential backoff
          const delay = Math.pow(2, this.maxRetries - config.retries) * 1000
          await new Promise(resolve => setTimeout(resolve, delay))

          return this.instance(config)
        }

        return Promise.reject(this.formatError(error))
      }
    )
  }

  private shouldRetry(error: any): boolean {
    // Retry on network errors or 5xx server errors
    return (
      !error.response ||
      (error.response.status >= 500 && error.response.status < 600) ||
      error.code === 'NETWORK_ERROR' ||
      error.code === 'TIMEOUT'
    )
  }

  private formatError(error: any): ApiError {
    const response = error.response
    const data = response?.data

    return {
      message: data?.message || error.message || 'An unexpected error occurred',
      code: data?.code || error.code || 'UNKNOWN_ERROR',
      details: data?.details || error.details,
      timestamp: new Date().toISOString()
    }
  }

  // HTTP Methods
  async get<T = any>(
    url: string,
    config: RequestConfig = {}
  ): Promise<ApiResponse<T>> {
    const response: AxiosResponse<ApiResponse<T>> = await this.instance.get(url, {
      ...config,
      retries: config.retries ?? this.maxRetries
    })
    return response.data
  }

  async post<T = any>(
    url: string,
    data?: any,
    config: RequestConfig = {}
  ): Promise<ApiResponse<T>> {
    const response: AxiosResponse<ApiResponse<T>> = await this.instance.post(url, data, {
      ...config,
      retries: config.retries ?? this.maxRetries
    })
    return response.data
  }

  async put<T = any>(
    url: string,
    data?: any,
    config: RequestConfig = {}
  ): Promise<ApiResponse<T>> {
    const response: AxiosResponse<ApiResponse<T>> = await this.instance.put(url, data, {
      ...config,
      retries: config.retries ?? this.maxRetries
    })
    return response.data
  }

  async patch<T = any>(
    url: string,
    data?: any,
    config: RequestConfig = {}
  ): Promise<ApiResponse<T>> {
    const response: AxiosResponse<ApiResponse<T>> = await this.instance.patch(url, data, {
      ...config,
      retries: config.retries ?? this.maxRetries
    })
    return response.data
  }

  async delete<T = any>(
    url: string,
    config: RequestConfig = {}
  ): Promise<ApiResponse<T>> {
    const response: AxiosResponse<ApiResponse<T>> = await this.instance.delete(url, {
      ...config,
      retries: config.retries ?? this.maxRetries
    })
    return response.data
  }

  // Paginated requests
  async getPaginated<T = any>(
    url: string,
    params: {
      page?: number
      limit?: number
      [key: string]: any
    } = {},
    config: RequestConfig = {}
  ): Promise<ApiResponse<PaginatedResponse<T>>> {
    return this.get<PaginatedResponse<T>>(url, {
      ...config,
      params: {
        page: 1,
        limit: 20,
        ...params
      }
    })
  }

  // File upload
  async upload<T = any>(
    url: string,
    file: File | FormData,
    config: RequestConfig = {}
  ): Promise<ApiResponse<T>> {
    const formData = file instanceof FormData ? file : new FormData()
    if (file instanceof File) {
      formData.append('file', file)
    }

    return this.post<T>(url, formData, {
      ...config,
      headers: {
        'Content-Type': 'multipart/form-data',
        ...config.headers
      }
    })
  }

  // Stream data (for real-time updates)
  createEventSource(
    url: string,
    options: {
      withCredentials?: boolean
      headers?: Record<string, string>
    } = {}
  ): EventSource {
    const fullUrl = new URL(url, this.baseURL).toString()

    // Add auth token to URL for EventSource (since it doesn't support custom headers)
    const authStore = useAuthStore()
    const token = authStore.token

    const urlWithAuth = token
      ? `${fullUrl}${fullUrl.includes('?') ? '&' : '?'}token=${encodeURIComponent(token)}`
      : fullUrl

    return new EventSource(urlWithAuth, {
      withCredentials: options.withCredentials ?? true
    })
  }

  // WebSocket connection
  createWebSocket(
    path: string,
    protocols?: string | string[]
  ): WebSocket {
    const wsUrl = this.baseURL.replace(/^https?/, 'ws')
    const fullUrl = new URL(path, wsUrl).toString()

    // Add auth token to URL
    const authStore = useAuthStore()
    const token = authStore.token

    const urlWithAuth = token
      ? `${fullUrl}${fullUrl.includes('?') ? '&' : '?'}token=${encodeURIComponent(token)}`
      : fullUrl

    return new WebSocket(urlWithAuth, protocols)
  }

  // Health check
  async healthCheck(): Promise<boolean> {
    try {
      await this.get('/health', { skipAuth: true, timeout: 5000, retries: 0 })
      return true
    } catch (error) {
      console.warn('API health check failed:', error)
      return false
    }
  }

  // Update base URL (useful for switching environments)
  updateBaseURL(newBaseURL: string): void {
    this.baseURL = newBaseURL
    this.instance.defaults.baseURL = newBaseURL
  }

  // Get current configuration
  getConfig() {
    return {
      baseURL: this.baseURL,
      timeout: this.defaultTimeout,
      maxRetries: this.maxRetries
    }
  }

  // Update configuration
  updateConfig(config: {
    timeout?: number
    maxRetries?: number
  }): void {
    if (config.timeout) {
      this.defaultTimeout = config.timeout
      this.instance.defaults.timeout = config.timeout
    }
    if (config.maxRetries !== undefined) {
      this.maxRetries = config.maxRetries
    }
  }
}

// Create and export singleton instance
export const apiClient = new ApiClient()

// Export class for testing or multiple instances
export { ApiClient }

// Utility functions
export const isApiError = (error: any): error is ApiError => {
  return error && typeof error.message === 'string' && typeof error.code === 'string'
}

export const handleApiError = (error: ApiError | any, defaultMessage = 'An error occurred') => {
  if (isApiError(error)) {
    return error.message
  }
  return error?.message || defaultMessage
}

// Request cancellation utility
export class CancelableRequest<T> {
  private controller: AbortController
  private promise: Promise<T>

  constructor(requestFn: (signal: AbortSignal) => Promise<T>) {
    this.controller = new AbortController()
    this.promise = requestFn(this.controller.signal)
  }

  get request(): Promise<T> {
    return this.promise
  }

  cancel(reason?: string): void {
    this.controller.abort(reason)
  }

  get signal(): AbortSignal {
    return this.controller.signal
  }
}

// Request queue for managing concurrent requests
export class RequestQueue {
  private queue: Array<() => Promise<any>> = []
  private running: Set<Promise<any>> = new Set()
  private maxConcurrent: number

  constructor(maxConcurrent: number = 5) {
    this.maxConcurrent = maxConcurrent
  }

  async add<T>(requestFn: () => Promise<T>): Promise<T> {
    return new Promise((resolve, reject) => {
      this.queue.push(async () => {
        try {
          const result = await requestFn()
          resolve(result)
        } catch (error) {
          reject(error)
        }
      })
      this.process()
    })
  }

  private async process(): Promise<void> {
    if (this.running.size >= this.maxConcurrent || this.queue.length === 0) {
      return
    }

    const requestFn = this.queue.shift()!
    const promise = requestFn()

    this.running.add(promise)

    try {
      await promise
    } finally {
      this.running.delete(promise)
      this.process() // Process next request
    }
  }

  clear(): void {
    this.queue.length = 0
  }

  get queueSize(): number {
    return this.queue.length
  }

  get activeRequests(): number {
    return this.running.size
  }
}

// Export default request queue instance
export const requestQueue = new RequestQueue()

// Rate limiting utility
export class RateLimiter {
  private requests: number[] = []
  private maxRequests: number
  private windowMs: number

  constructor(maxRequests: number = 100, windowMs: number = 60000) {
    this.maxRequests = maxRequests
    this.windowMs = windowMs
  }

  async checkLimit(): Promise<boolean> {
    const now = Date.now()

    // Remove old requests outside the window
    this.requests = this.requests.filter(time => now - time < this.windowMs)

    if (this.requests.length >= this.maxRequests) {
      return false
    }

    this.requests.push(now)
    return true
  }

  getRemainingRequests(): number {
    const now = Date.now()
    this.requests = this.requests.filter(time => now - time < this.windowMs)
    return Math.max(0, this.maxRequests - this.requests.length)
  }

  getResetTime(): number {
    if (this.requests.length === 0) return 0
    return this.requests[0] + this.windowMs
  }
}

// Default rate limiter instance
export const rateLimiter = new RateLimiter()