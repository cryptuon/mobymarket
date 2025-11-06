import { apiClient } from './base'
import type { ApiResponse, PaginatedResponse } from './base'

/**
 * Portfolio management API service
 */

export interface Portfolio {
  id: string
  name: string
  description?: string
  type: 'personal' | 'business' | 'demo'
  currency: string
  totalValue: number
  totalCost: number
  totalPnL: number
  change24h: number
  change7d: number
  change30d: number
  diversityScore: number
  riskLevel: 'low' | 'moderate' | 'high'
  positions: Position[]
  settings: PortfolioSettings
  createdAt: string
  updatedAt: string
  lastActivityAt: string
}

export interface Position {
  id: string
  portfolioId: string
  symbol: string
  name: string
  amount: number
  averagePrice: number
  currentPrice: number
  totalValue: number
  totalCost: number
  unrealizedPnL: number
  realizedPnL: number
  allocation: number
  status: 'active' | 'closed' | 'partial'
  firstPurchaseAt: string
  lastTransactionAt: string
  transactions: Transaction[]
  metadata?: {
    tokenAddress?: string
    network?: string
    category?: string
    tags?: string[]
  }
}

export interface Transaction {
  id: string
  portfolioId: string
  positionId?: string
  type: 'buy' | 'sell' | 'swap' | 'transfer' | 'deposit' | 'withdrawal'
  symbol: string
  amount: number
  price: number
  value: number
  fee: number
  notes?: string
  status: 'pending' | 'completed' | 'failed' | 'cancelled'
  txHash?: string
  blockNumber?: number
  network?: string
  timestamp: string
  createdAt: string
}

export interface PortfolioSettings {
  autoSync: boolean
  rebalanceThreshold: number
  riskTolerance: 'conservative' | 'moderate' | 'aggressive'
  notifications: {
    priceAlerts: boolean
    rebalanceAlerts: boolean
    largeMovements: boolean
  }
  privacy: {
    public: boolean
    shareable: boolean
  }
  integration: {
    walletAddresses: string[]
    exchangeAccounts: string[]
    apiKeys: string[]
  }
}

export interface PortfolioSummary {
  id: string
  name: string
  totalValue: number
  change24h: number
  positionCount: number
  lastUpdate: string
}

export interface PerformanceMetrics {
  portfolioId: string
  period: '24h' | '7d' | '30d' | '90d' | '1y' | 'all'
  totalReturn: number
  totalReturnPercent: number
  volatility: number
  sharpeRatio: number
  sortinoRatio: number
  maxDrawdown: number
  winRate: number
  bestDay: number
  worstDay: number
  benchmark?: {
    name: string
    return: number
    correlation: number
  }
  chartData: Array<{
    date: string
    value: number
    benchmark?: number
  }>
}

export interface AllocationData {
  portfolioId: string
  byAsset: Array<{
    symbol: string
    name: string
    value: number
    percentage: number
    color: string
  }>
  byCategory: Array<{
    category: string
    value: number
    percentage: number
    assets: string[]
  }>
  byNetwork: Array<{
    network: string
    value: number
    percentage: number
    assets: string[]
  }>
}

export interface RiskMetrics {
  portfolioId: string
  riskScore: number
  var95: number
  expectedShortfall: number
  beta: number
  correlations: Array<{
    asset: string
    correlation: number
  }>
  concentrationRisk: number
  liquidityRisk: number
  volatilityRisk: number
}

export interface RebalanceRecommendation {
  portfolioId: string
  targetAllocations: Array<{
    symbol: string
    currentPercentage: number
    targetPercentage: number
    action: 'buy' | 'sell' | 'hold'
    amount: number
    value: number
  }>
  estimatedCost: number
  estimatedTax: number
  riskImprovement: number
  diversityImprovement: number
  reasoning: string[]
}

export interface PortfolioComparison {
  portfolios: Array<{
    id: string
    name: string
    totalValue: number
    totalReturn: number
    returnPercent: number
    volatility: number
    sharpeRatio: number
    maxDrawdown: number
  }>
  period: string
  benchmark?: {
    name: string
    return: number
  }
}

class PortfolioService {
  private readonly basePath = '/portfolios'

  /**
   * Get all portfolios for the user
   */
  async getPortfolios(): Promise<ApiResponse<PortfolioSummary[]>> {
    return apiClient.get(`${this.basePath}`)
  }

  /**
   * Get a specific portfolio by ID
   */
  async getPortfolio(id: string): Promise<ApiResponse<Portfolio>> {
    return apiClient.get(`${this.basePath}/${id}`)
  }

  /**
   * Create a new portfolio
   */
  async createPortfolio(data: {
    name: string
    description?: string
    type?: Portfolio['type']
    currency?: string
    settings?: Partial<PortfolioSettings>
  }): Promise<ApiResponse<Portfolio>> {
    return apiClient.post(`${this.basePath}`, data)
  }

  /**
   * Update portfolio details
   */
  async updatePortfolio(id: string, data: Partial<Portfolio>): Promise<ApiResponse<Portfolio>> {
    return apiClient.patch(`${this.basePath}/${id}`, data)
  }

  /**
   * Delete a portfolio
   */
  async deletePortfolio(id: string): Promise<ApiResponse<void>> {
    return apiClient.delete(`${this.basePath}/${id}`)
  }

  /**
   * Duplicate a portfolio
   */
  async duplicatePortfolio(id: string, name: string): Promise<ApiResponse<Portfolio>> {
    return apiClient.post(`${this.basePath}/${id}/duplicate`, { name })
  }

  // Positions

  /**
   * Get positions for a portfolio
   */
  async getPositions(portfolioId: string, params?: {
    status?: Position['status']
    symbol?: string
    page?: number
    limit?: number
  }): Promise<ApiResponse<PaginatedResponse<Position>>> {
    return apiClient.getPaginated(`${this.basePath}/${portfolioId}/positions`, params)
  }

  /**
   * Get a specific position
   */
  async getPosition(portfolioId: string, positionId: string): Promise<ApiResponse<Position>> {
    return apiClient.get(`${this.basePath}/${portfolioId}/positions/${positionId}`)
  }

  /**
   * Add a new position
   */
  async addPosition(portfolioId: string, data: {
    symbol: string
    amount: number
    price: number
    date?: string
    notes?: string
    metadata?: Position['metadata']
  }): Promise<ApiResponse<Position>> {
    return apiClient.post(`${this.basePath}/${portfolioId}/positions`, data)
  }

  /**
   * Update a position
   */
  async updatePosition(
    portfolioId: string,
    positionId: string,
    data: Partial<Position>
  ): Promise<ApiResponse<Position>> {
    return apiClient.patch(`${this.basePath}/${portfolioId}/positions/${positionId}`, data)
  }

  /**
   * Close a position
   */
  async closePosition(
    portfolioId: string,
    positionId: string,
    data: {
      price: number
      date?: string
      notes?: string
    }
  ): Promise<ApiResponse<Position>> {
    return apiClient.post(`${this.basePath}/${portfolioId}/positions/${positionId}/close`, data)
  }

  /**
   * Delete a position
   */
  async deletePosition(portfolioId: string, positionId: string): Promise<ApiResponse<void>> {
    return apiClient.delete(`${this.basePath}/${portfolioId}/positions/${positionId}`)
  }

  // Transactions

  /**
   * Get transactions for a portfolio
   */
  async getTransactions(portfolioId: string, params?: {
    type?: Transaction['type']
    symbol?: string
    status?: Transaction['status']
    startDate?: string
    endDate?: string
    page?: number
    limit?: number
  }): Promise<ApiResponse<PaginatedResponse<Transaction>>> {
    return apiClient.getPaginated(`${this.basePath}/${portfolioId}/transactions`, params)
  }

  /**
   * Get a specific transaction
   */
  async getTransaction(portfolioId: string, transactionId: string): Promise<ApiResponse<Transaction>> {
    return apiClient.get(`${this.basePath}/${portfolioId}/transactions/${transactionId}`)
  }

  /**
   * Add a new transaction
   */
  async addTransaction(portfolioId: string, data: {
    type: Transaction['type']
    symbol: string
    amount: number
    price: number
    fee?: number
    timestamp?: string
    notes?: string
    txHash?: string
    network?: string
  }): Promise<ApiResponse<Transaction>> {
    return apiClient.post(`${this.basePath}/${portfolioId}/transactions`, data)
  }

  /**
   * Update a transaction
   */
  async updateTransaction(
    portfolioId: string,
    transactionId: string,
    data: Partial<Transaction>
  ): Promise<ApiResponse<Transaction>> {
    return apiClient.patch(`${this.basePath}/${portfolioId}/transactions/${transactionId}`, data)
  }

  /**
   * Delete a transaction
   */
  async deleteTransaction(portfolioId: string, transactionId: string): Promise<ApiResponse<void>> {
    return apiClient.delete(`${this.basePath}/${portfolioId}/transactions/${transactionId}`)
  }

  /**
   * Import transactions from CSV
   */
  async importTransactions(portfolioId: string, file: File): Promise<ApiResponse<{
    imported: number
    skipped: number
    errors: Array<{ row: number, error: string }>
  }>> {
    return apiClient.upload(`${this.basePath}/${portfolioId}/transactions/import`, file)
  }

  /**
   * Export transactions to CSV
   */
  async exportTransactions(portfolioId: string, format: 'csv' | 'xlsx' = 'csv'): Promise<ApiResponse<{
    downloadUrl: string
    expiresAt: string
  }>> {
    return apiClient.post(`${this.basePath}/${portfolioId}/transactions/export`, { format })
  }

  // Analytics

  /**
   * Get portfolio performance metrics
   */
  async getPerformance(portfolioId: string, period: PerformanceMetrics['period'] = '30d'): Promise<ApiResponse<PerformanceMetrics>> {
    return apiClient.get(`${this.basePath}/${portfolioId}/performance`, {
      params: { period }
    })
  }

  /**
   * Get portfolio allocation data
   */
  async getAllocation(portfolioId: string): Promise<ApiResponse<AllocationData>> {
    return apiClient.get(`${this.basePath}/${portfolioId}/allocation`)
  }

  /**
   * Get portfolio risk metrics
   */
  async getRiskMetrics(portfolioId: string): Promise<ApiResponse<RiskMetrics>> {
    return apiClient.get(`${this.basePath}/${portfolioId}/risk`)
  }

  /**
   * Get rebalancing recommendations
   */
  async getRebalanceRecommendations(portfolioId: string, params?: {
    strategy?: 'conservative' | 'balanced' | 'aggressive'
    threshold?: number
    maxPositions?: number
  }): Promise<ApiResponse<RebalanceRecommendation>> {
    return apiClient.get(`${this.basePath}/${portfolioId}/rebalance`, {
      params
    })
  }

  /**
   * Execute portfolio rebalancing
   */
  async executeRebalance(portfolioId: string, data: {
    targetAllocations: Array<{
      symbol: string
      targetPercentage: number
    }>
    maxSlippage?: number
    dryRun?: boolean
  }): Promise<ApiResponse<{
    transactions: Transaction[]
    estimatedCost: number
    success: boolean
  }>> {
    return apiClient.post(`${this.basePath}/${portfolioId}/rebalance/execute`, data)
  }

  /**
   * Compare multiple portfolios
   */
  async comparePortfolios(portfolioIds: string[], period: string = '30d'): Promise<ApiResponse<PortfolioComparison>> {
    return apiClient.post(`${this.basePath}/compare`, {
      portfolioIds,
      period
    })
  }

  // Sync and Integration

  /**
   * Sync portfolio with external wallet
   */
  async syncWallet(portfolioId: string, data: {
    address: string
    network: string
    type: 'evm' | 'bitcoin' | 'solana'
  }): Promise<ApiResponse<{
    syncedTransactions: number
    newPositions: number
    updatedPositions: number
  }>> {
    return apiClient.post(`${this.basePath}/${portfolioId}/sync/wallet`, data)
  }

  /**
   * Sync portfolio with exchange
   */
  async syncExchange(portfolioId: string, data: {
    exchange: string
    apiKey: string
    apiSecret: string
    sandbox?: boolean
  }): Promise<ApiResponse<{
    syncedTransactions: number
    newPositions: number
    updatedPositions: number
  }>> {
    return apiClient.post(`${this.basePath}/${portfolioId}/sync/exchange`, data)
  }

  /**
   * Get sync status
   */
  async getSyncStatus(portfolioId: string): Promise<ApiResponse<{
    lastSync: string
    status: 'syncing' | 'success' | 'error' | 'never'
    error?: string
    wallets: Array<{
      address: string
      network: string
      lastSync: string
      status: string
    }>
    exchanges: Array<{
      exchange: string
      lastSync: string
      status: string
    }>
  }>> {
    return apiClient.get(`${this.basePath}/${portfolioId}/sync/status`)
  }

  // Price Alerts

  /**
   * Get price alerts for portfolio
   */
  async getPriceAlerts(portfolioId: string): Promise<ApiResponse<Array<{
    id: string
    symbol: string
    type: 'above' | 'below' | 'change'
    value: number
    active: boolean
    triggered: boolean
    createdAt: string
  }>>> {
    return apiClient.get(`${this.basePath}/${portfolioId}/alerts`)
  }

  /**
   * Create price alert
   */
  async createPriceAlert(portfolioId: string, data: {
    symbol: string
    type: 'above' | 'below' | 'change'
    value: number
    notification: 'email' | 'push' | 'both'
  }): Promise<ApiResponse<void>> {
    return apiClient.post(`${this.basePath}/${portfolioId}/alerts`, data)
  }

  /**
   * Delete price alert
   */
  async deletePriceAlert(portfolioId: string, alertId: string): Promise<ApiResponse<void>> {
    return apiClient.delete(`${this.basePath}/${portfolioId}/alerts/${alertId}`)
  }

  // Sharing and Collaboration

  /**
   * Share portfolio
   */
  async sharePortfolio(portfolioId: string, data: {
    type: 'public' | 'private'
    permissions: ('view' | 'comment')[]
    expiresAt?: string
  }): Promise<ApiResponse<{
    shareUrl: string
    shareId: string
  }>> {
    return apiClient.post(`${this.basePath}/${portfolioId}/share`, data)
  }

  /**
   * Get shared portfolio
   */
  async getSharedPortfolio(shareId: string): Promise<ApiResponse<Portfolio>> {
    return apiClient.get(`/shared/portfolios/${shareId}`, { skipAuth: true })
  }

  /**
   * Revoke portfolio share
   */
  async revokeShare(portfolioId: string, shareId: string): Promise<ApiResponse<void>> {
    return apiClient.delete(`${this.basePath}/${portfolioId}/share/${shareId}`)
  }

  // Real-time Updates

  /**
   * Subscribe to portfolio updates
   */
  subscribeToUpdates(portfolioId: string, callbacks: {
    onPositionUpdate?: (position: Position) => void
    onTransactionUpdate?: (transaction: Transaction) => void
    onPortfolioUpdate?: (portfolio: Portfolio) => void
    onPriceAlert?: (alert: any) => void
  }): WebSocket {
    const ws = apiClient.createWebSocket(`/portfolios/${portfolioId}/subscribe`)

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data)

      switch (data.type) {
        case 'position_update':
          callbacks.onPositionUpdate?.(data.payload)
          break
        case 'transaction_update':
          callbacks.onTransactionUpdate?.(data.payload)
          break
        case 'portfolio_update':
          callbacks.onPortfolioUpdate?.(data.payload)
          break
        case 'price_alert':
          callbacks.onPriceAlert?.(data.payload)
          break
      }
    }

    return ws
  }
}

// Export singleton instance
export const portfolioService = new PortfolioService()

// Export class for testing
export { PortfolioService }