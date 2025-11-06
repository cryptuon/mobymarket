import { apiClient } from './base'
import type { ApiResponse, PaginatedResponse } from './base'

/**
 * Trading and order management API service
 */

export interface TradingAccount {
  id: string
  name: string
  exchange: string
  type: 'spot' | 'margin' | 'futures' | 'options'
  status: 'active' | 'suspended' | 'maintenance'
  balances: Record<string, {
    available: number
    locked: number
    total: number
    usdValue: number
  }>
  tradingPermissions: {
    spot: boolean
    margin: boolean
    futures: boolean
    options: boolean
  }
  riskLimits: {
    maxOrderSize: number
    maxPositionSize: number
    maxDailyLoss: number
    marginLevel: number
  }
  fees: {
    maker: number
    taker: number
    withdrawal: Record<string, number>
  }
  lastUpdated: string
}

export interface Order {
  id: string
  clientOrderId?: string
  symbol: string
  side: 'buy' | 'sell'
  type: 'market' | 'limit' | 'stop' | 'stop_limit' | 'trailing_stop'
  status: 'pending' | 'open' | 'partially_filled' | 'filled' | 'cancelled' | 'rejected' | 'expired'
  timeInForce: 'GTC' | 'IOC' | 'FOK' | 'GTD'
  quantity: number
  price?: number
  stopPrice?: number
  trailingAmount?: number
  executedQuantity: number
  executedPrice: number
  remainingQuantity: number
  commission: number
  commissionAsset: string
  createdAt: string
  updatedAt: string
  fills: OrderFill[]
}

export interface OrderFill {
  id: string
  price: number
  quantity: number
  commission: number
  commissionAsset: string
  timestamp: string
}

export interface Position {
  symbol: string
  side: 'long' | 'short'
  size: number
  entryPrice: number
  markPrice: number
  unrealizedPnL: number
  realizedPnL: number
  percentage: number
  margin: number
  marginRatio: number
  liquidationPrice?: number
  timestamp: string
}

export interface TradingPair {
  symbol: string
  baseAsset: string
  quoteAsset: string
  status: 'trading' | 'halt' | 'break'
  baseAssetPrecision: number
  quotePrecision: number
  pricePrecision: number
  quantityPrecision: number
  minOrderSize: number
  maxOrderSize: number
  minPrice: number
  maxPrice: number
  tickSize: number
  stepSize: number
  fees: {
    maker: number
    taker: number
  }
}

export interface OrderBook {
  symbol: string
  lastUpdateId: number
  bids: Array<[string, string]> // [price, quantity]
  asks: Array<[string, string]> // [price, quantity]
  timestamp: string
}

export interface Trade {
  id: string
  symbol: string
  price: number
  quantity: number
  side: 'buy' | 'sell'
  timestamp: string
  isBuyerMaker: boolean
}

export interface Ticker {
  symbol: string
  price: number
  priceChange: number
  priceChangePercent: number
  weightedAvgPrice: number
  prevClosePrice: number
  lastPrice: number
  lastQty: number
  bidPrice: number
  bidQty: number
  askPrice: number
  askQty: number
  openPrice: number
  highPrice: number
  lowPrice: number
  volume: number
  quoteVolume: number
  openTime: string
  closeTime: string
  count: number
}

export interface Kline {
  symbol: string
  interval: '1m' | '3m' | '5m' | '15m' | '30m' | '1h' | '2h' | '4h' | '6h' | '8h' | '12h' | '1d' | '3d' | '1w' | '1M'
  openTime: string
  closeTime: string
  open: number
  high: number
  low: number
  close: number
  volume: number
  quoteVolume: number
  trades: number
  takerBuyBaseVolume: number
  takerBuyQuoteVolume: number
}

export interface TradingStrategy {
  id: string
  name: string
  description: string
  type: 'dca' | 'grid' | 'arbitrage' | 'momentum' | 'mean_reversion' | 'custom'
  status: 'active' | 'paused' | 'stopped'
  parameters: Record<string, any>
  performance: {
    totalReturn: number
    returnPercent: number
    sharpeRatio: number
    maxDrawdown: number
    winRate: number
    totalTrades: number
  }
  settings: {
    maxInvestment: number
    riskPerTrade: number
    stopLoss?: number
    takeProfit?: number
    trailing?: boolean
  }
  createdAt: string
  updatedAt: string
}

export interface TradingSignal {
  id: string
  symbol: string
  type: 'buy' | 'sell'
  strength: 'weak' | 'moderate' | 'strong'
  confidence: number
  source: 'technical' | 'fundamental' | 'sentiment' | 'algorithm'
  indicators: Array<{
    name: string
    value: number
    signal: 'buy' | 'sell' | 'neutral'
  }>
  timeframe: string
  targetPrice?: number
  stopLoss?: number
  expiresAt: string
  createdAt: string
}

export interface RiskAssessment {
  portfolioValue: number
  exposure: Record<string, number>
  var95: number
  expectedShortfall: number
  maxDrawdown: number
  sharpeRatio: number
  riskScore: number
  recommendations: Array<{
    type: 'warning' | 'suggestion' | 'critical'
    message: string
    action?: string
  }>
}

class TradingService {
  private readonly basePath = '/trading'

  // Account Management

  /**
   * Get trading accounts
   */
  async getAccounts(): Promise<ApiResponse<TradingAccount[]>> {
    return apiClient.get(`${this.basePath}/accounts`)
  }

  /**
   * Get specific trading account
   */
  async getAccount(accountId: string): Promise<ApiResponse<TradingAccount>> {
    return apiClient.get(`${this.basePath}/accounts/${accountId}`)
  }

  /**
   * Connect new trading account
   */
  async connectAccount(data: {
    exchange: string
    apiKey: string
    apiSecret: string
    passphrase?: string
    sandbox?: boolean
    name?: string
  }): Promise<ApiResponse<TradingAccount>> {
    return apiClient.post(`${this.basePath}/accounts`, data)
  }

  /**
   * Update account settings
   */
  async updateAccount(accountId: string, data: Partial<TradingAccount>): Promise<ApiResponse<TradingAccount>> {
    return apiClient.patch(`${this.basePath}/accounts/${accountId}`, data)
  }

  /**
   * Delete trading account
   */
  async deleteAccount(accountId: string): Promise<ApiResponse<void>> {
    return apiClient.delete(`${this.basePath}/accounts/${accountId}`)
  }

  /**
   * Get account balances
   */
  async getBalances(accountId: string): Promise<ApiResponse<Record<string, {
    available: number
    locked: number
    total: number
    usdValue: number
  }>>> {
    return apiClient.get(`${this.basePath}/accounts/${accountId}/balances`)
  }

  // Order Management

  /**
   * Get orders
   */
  async getOrders(accountId: string, params?: {
    symbol?: string
    status?: Order['status']
    side?: Order['side']
    type?: Order['type']
    startTime?: string
    endTime?: string
    limit?: number
    offset?: number
  }): Promise<ApiResponse<PaginatedResponse<Order>>> {
    return apiClient.getPaginated(`${this.basePath}/accounts/${accountId}/orders`, params)
  }

  /**
   * Get specific order
   */
  async getOrder(accountId: string, orderId: string): Promise<ApiResponse<Order>> {
    return apiClient.get(`${this.basePath}/accounts/${accountId}/orders/${orderId}`)
  }

  /**
   * Place new order
   */
  async placeOrder(accountId: string, data: {
    symbol: string
    side: Order['side']
    type: Order['type']
    quantity: number
    price?: number
    stopPrice?: number
    trailingAmount?: number
    timeInForce?: Order['timeInForce']
    clientOrderId?: string
    reduceOnly?: boolean
    closePosition?: boolean
  }): Promise<ApiResponse<Order>> {
    return apiClient.post(`${this.basePath}/accounts/${accountId}/orders`, data)
  }

  /**
   * Cancel order
   */
  async cancelOrder(accountId: string, orderId: string): Promise<ApiResponse<Order>> {
    return apiClient.delete(`${this.basePath}/accounts/${accountId}/orders/${orderId}`)
  }

  /**
   * Cancel all orders for symbol
   */
  async cancelAllOrders(accountId: string, symbol?: string): Promise<ApiResponse<{
    cancelled: number
    orders: Order[]
  }>> {
    return apiClient.delete(`${this.basePath}/accounts/${accountId}/orders`, {
      params: { symbol }
    })
  }

  /**
   * Modify order
   */
  async modifyOrder(accountId: string, orderId: string, data: {
    quantity?: number
    price?: number
    stopPrice?: number
  }): Promise<ApiResponse<Order>> {
    return apiClient.patch(`${this.basePath}/accounts/${accountId}/orders/${orderId}`, data)
  }

  // Positions

  /**
   * Get open positions
   */
  async getPositions(accountId: string, symbol?: string): Promise<ApiResponse<Position[]>> {
    return apiClient.get(`${this.basePath}/accounts/${accountId}/positions`, {
      params: { symbol }
    })
  }

  /**
   * Close position
   */
  async closePosition(accountId: string, symbol: string, data?: {
    quantity?: number
    price?: number
    type?: 'market' | 'limit'
  }): Promise<ApiResponse<Order>> {
    return apiClient.post(`${this.basePath}/accounts/${accountId}/positions/${symbol}/close`, data)
  }

  /**
   * Update position margin
   */
  async updatePositionMargin(accountId: string, symbol: string, data: {
    amount: number
    type: 'add' | 'reduce'
  }): Promise<ApiResponse<Position>> {
    return apiClient.post(`${this.basePath}/accounts/${accountId}/positions/${symbol}/margin`, data)
  }

  // Market Data

  /**
   * Get trading pairs
   */
  async getTradingPairs(exchange?: string): Promise<ApiResponse<TradingPair[]>> {
    return apiClient.get(`${this.basePath}/pairs`, {
      params: { exchange }
    })
  }

  /**
   * Get order book
   */
  async getOrderBook(symbol: string, depth: number = 100): Promise<ApiResponse<OrderBook>> {
    return apiClient.get(`${this.basePath}/orderbook/${symbol}`, {
      params: { depth }
    })
  }

  /**
   * Get recent trades
   */
  async getRecentTrades(symbol: string, limit: number = 100): Promise<ApiResponse<Trade[]>> {
    return apiClient.get(`${this.basePath}/trades/${symbol}`, {
      params: { limit }
    })
  }

  /**
   * Get ticker information
   */
  async getTicker(symbol?: string): Promise<ApiResponse<Ticker | Ticker[]>> {
    const endpoint = symbol ? `${this.basePath}/ticker/${symbol}` : `${this.basePath}/ticker`
    return apiClient.get(endpoint)
  }

  /**
   * Get kline/candlestick data
   */
  async getKlines(symbol: string, interval: Kline['interval'], params?: {
    startTime?: string
    endTime?: string
    limit?: number
  }): Promise<ApiResponse<Kline[]>> {
    return apiClient.get(`${this.basePath}/klines/${symbol}`, {
      params: {
        interval,
        ...params
      }
    })
  }

  // Trading Strategies

  /**
   * Get trading strategies
   */
  async getStrategies(): Promise<ApiResponse<TradingStrategy[]>> {
    return apiClient.get(`${this.basePath}/strategies`)
  }

  /**
   * Get specific strategy
   */
  async getStrategy(strategyId: string): Promise<ApiResponse<TradingStrategy & {
    trades: Array<{
      id: string
      symbol: string
      side: 'buy' | 'sell'
      quantity: number
      price: number
      pnl: number
      timestamp: string
    }>
    performance: {
      daily: Array<{
        date: string
        value: number
        return: number
      }>
      metrics: Record<string, number>
    }
  }>> {
    return apiClient.get(`${this.basePath}/strategies/${strategyId}`)
  }

  /**
   * Create new strategy
   */
  async createStrategy(data: {
    name: string
    description?: string
    type: TradingStrategy['type']
    parameters: Record<string, any>
    settings: TradingStrategy['settings']
  }): Promise<ApiResponse<TradingStrategy>> {
    return apiClient.post(`${this.basePath}/strategies`, data)
  }

  /**
   * Update strategy
   */
  async updateStrategy(strategyId: string, data: Partial<TradingStrategy>): Promise<ApiResponse<TradingStrategy>> {
    return apiClient.patch(`${this.basePath}/strategies/${strategyId}`, data)
  }

  /**
   * Start/Resume strategy
   */
  async startStrategy(strategyId: string): Promise<ApiResponse<void>> {
    return apiClient.post(`${this.basePath}/strategies/${strategyId}/start`)
  }

  /**
   * Pause strategy
   */
  async pauseStrategy(strategyId: string): Promise<ApiResponse<void>> {
    return apiClient.post(`${this.basePath}/strategies/${strategyId}/pause`)
  }

  /**
   * Stop strategy
   */
  async stopStrategy(strategyId: string): Promise<ApiResponse<void>> {
    return apiClient.post(`${this.basePath}/strategies/${strategyId}/stop`)
  }

  /**
   * Delete strategy
   */
  async deleteStrategy(strategyId: string): Promise<ApiResponse<void>> {
    return apiClient.delete(`${this.basePath}/strategies/${strategyId}`)
  }

  /**
   * Backtest strategy
   */
  async backtestStrategy(data: {
    strategy: Omit<TradingStrategy, 'id' | 'status' | 'performance' | 'createdAt' | 'updatedAt'>
    startDate: string
    endDate: string
    initialCapital: number
    symbols?: string[]
  }): Promise<ApiResponse<{
    performance: {
      totalReturn: number
      returnPercent: number
      sharpeRatio: number
      maxDrawdown: number
      winRate: number
      totalTrades: number
      profitFactor: number
      avgWin: number
      avgLoss: number
    }
    trades: Array<{
      date: string
      symbol: string
      side: 'buy' | 'sell'
      quantity: number
      price: number
      pnl: number
    }>
    equity: Array<{
      date: string
      value: number
    }>
  }>> {
    return apiClient.post(`${this.basePath}/strategies/backtest`, data)
  }

  // Trading Signals

  /**
   * Get trading signals
   */
  async getSignals(params?: {
    symbol?: string
    type?: TradingSignal['type']
    strength?: TradingSignal['strength']
    source?: TradingSignal['source']
    active?: boolean
    limit?: number
  }): Promise<ApiResponse<TradingSignal[]>> {
    return apiClient.get(`${this.basePath}/signals`, { params })
  }

  /**
   * Create custom signal
   */
  async createSignal(data: {
    symbol: string
    type: TradingSignal['type']
    strength: TradingSignal['strength']
    confidence: number
    indicators: TradingSignal['indicators']
    timeframe: string
    targetPrice?: number
    stopLoss?: number
    expiresAt: string
    notes?: string
  }): Promise<ApiResponse<TradingSignal>> {
    return apiClient.post(`${this.basePath}/signals`, data)
  }

  /**
   * Delete signal
   */
  async deleteSignal(signalId: string): Promise<ApiResponse<void>> {
    return apiClient.delete(`${this.basePath}/signals/${signalId}`)
  }

  // Risk Management

  /**
   * Get risk assessment
   */
  async getRiskAssessment(accountId: string): Promise<ApiResponse<RiskAssessment>> {
    return apiClient.get(`${this.basePath}/accounts/${accountId}/risk`)
  }

  /**
   * Update risk settings
   */
  async updateRiskSettings(accountId: string, data: {
    maxOrderSize?: number
    maxPositionSize?: number
    maxDailyLoss?: number
    stopLossPercent?: number
    takeProfitPercent?: number
    maxLeverage?: number
  }): Promise<ApiResponse<void>> {
    return apiClient.patch(`${this.basePath}/accounts/${accountId}/risk`, data)
  }

  /**
   * Calculate position size
   */
  async calculatePositionSize(data: {
    accountId: string
    symbol: string
    riskAmount: number
    entryPrice: number
    stopLoss: number
  }): Promise<ApiResponse<{
    quantity: number
    value: number
    riskReward: number
    marginRequired: number
  }>> {
    return apiClient.post(`${this.basePath}/position-size`, data)
  }

  // Analytics

  /**
   * Get trading performance
   */
  async getTradingPerformance(accountId: string, period: string = '30d'): Promise<ApiResponse<{
    totalReturn: number
    returnPercent: number
    totalTrades: number
    winningTrades: number
    losingTrades: number
    winRate: number
    averageWin: number
    averageLoss: number
    profitFactor: number
    sharpeRatio: number
    maxDrawdown: number
    dailyReturns: Array<{
      date: string
      return: number
      cumulative: number
    }>
  }>> {
    return apiClient.get(`${this.basePath}/accounts/${accountId}/performance`, {
      params: { period }
    })
  }

  /**
   * Get trade history
   */
  async getTradeHistory(accountId: string, params?: {
    symbol?: string
    startDate?: string
    endDate?: string
    limit?: number
    offset?: number
  }): Promise<ApiResponse<PaginatedResponse<{
    id: string
    symbol: string
    side: 'buy' | 'sell'
    quantity: number
    price: number
    value: number
    fee: number
    pnl: number
    timestamp: string
  }>>> {
    return apiClient.getPaginated(`${this.basePath}/accounts/${accountId}/trades`, params)
  }

  // Real-time Data Streams

  /**
   * Subscribe to order updates
   */
  subscribeToOrders(
    accountId: string,
    callback: (order: Order) => void
  ): WebSocket {
    const ws = apiClient.createWebSocket(`/trading/accounts/${accountId}/orders/subscribe`)

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data)
      if (data.type === 'order_update') {
        callback(data.payload)
      }
    }

    return ws
  }

  /**
   * Subscribe to position updates
   */
  subscribeToPositions(
    accountId: string,
    callback: (position: Position) => void
  ): WebSocket {
    const ws = apiClient.createWebSocket(`/trading/accounts/${accountId}/positions/subscribe`)

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data)
      if (data.type === 'position_update') {
        callback(data.payload)
      }
    }

    return ws
  }

  /**
   * Subscribe to ticker updates
   */
  subscribeToTicker(
    symbols: string[],
    callback: (ticker: Ticker) => void
  ): WebSocket {
    const ws = apiClient.createWebSocket(`/trading/ticker/subscribe`)

    ws.onopen = () => {
      ws.send(JSON.stringify({
        action: 'subscribe',
        symbols
      }))
    }

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data)
      if (data.type === 'ticker_update') {
        callback(data.payload)
      }
    }

    return ws
  }

  /**
   * Subscribe to order book updates
   */
  subscribeToOrderBook(
    symbol: string,
    callback: (orderBook: OrderBook) => void
  ): WebSocket {
    const ws = apiClient.createWebSocket(`/trading/orderbook/${symbol}/subscribe`)

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data)
      if (data.type === 'orderbook_update') {
        callback(data.payload)
      }
    }

    return ws
  }

  /**
   * Subscribe to trade updates
   */
  subscribeToTrades(
    symbol: string,
    callback: (trade: Trade) => void
  ): WebSocket {
    const ws = apiClient.createWebSocket(`/trading/trades/${symbol}/subscribe`)

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data)
      if (data.type === 'trade_update') {
        callback(data.payload)
      }
    }

    return ws
  }
}

// Export singleton instance
export const tradingService = new TradingService()

// Export class for testing
export { TradingService }