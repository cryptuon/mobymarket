import { apiClient } from './base'
import type { ApiResponse, PaginatedResponse } from './base'

/**
 * Market data and analytics API service
 */

export interface Asset {
  id: string
  symbol: string
  name: string
  image: string
  currentPrice: number
  marketCap: number
  marketCapRank: number
  fullyDilutedValuation?: number
  totalVolume: number
  high24h: number
  low24h: number
  priceChange24h: number
  priceChangePercentage24h: number
  priceChangePercentage7d: number
  priceChangePercentage30d: number
  circulatingSupply: number
  totalSupply?: number
  maxSupply?: number
  ath: number
  athChangePercentage: number
  athDate: string
  atl: number
  atlChangePercentage: number
  atlDate: string
  lastUpdated: string
  sparklineIn7d?: {
    price: number[]
  }
  roi?: {
    times: number
    currency: string
    percentage: number
  }
}

export interface PriceData {
  symbol: string
  price: number
  timestamp: string
  change24h: number
  changePercent24h: number
  volume24h: number
  marketCap: number
}

export interface ChartData {
  symbol: string
  timeframe: '1m' | '5m' | '15m' | '1h' | '4h' | '1d' | '7d' | '30d' | '90d' | '1y'
  data: Array<{
    timestamp: number
    open: number
    high: number
    low: number
    close: number
    volume: number
  }>
}

export interface MarketStats {
  totalMarketCap: number
  totalVolume: number
  marketCapChange24h: number
  volumeChange24h: number
  btcDominance: number
  ethDominance: number
  defiTvl: number
  activeCoins: number
  markets: number
  fearGreedIndex: {
    value: number
    classification: string
    timestamp: string
  }
}

export interface TrendingAsset {
  id: string
  symbol: string
  name: string
  image: string
  rank: number
  priceChange24h: number
  volume: number
  sparkline: number[]
}

export interface WhaleTransaction {
  id: string
  hash: string
  symbol: string
  amount: number
  usdValue: number
  from: string
  to: string
  type: 'transfer' | 'deposit' | 'withdrawal' | 'swap'
  exchange?: string
  timestamp: string
  blockNumber?: number
  network: string
}

export interface MarketSentiment {
  overall: number
  news: number
  social: number
  onChain: number
  technical: number
  components: {
    fearGreed: number
    volatility: number
    momentum: number
    volume: number
    marketCap: number
  }
  lastUpdated: string
}

export interface NewsArticle {
  id: string
  title: string
  content: string
  summary: string
  url: string
  source: string
  author?: string
  publishedAt: string
  sentiment: 'positive' | 'neutral' | 'negative'
  relevantSymbols: string[]
  categories: string[]
  image?: string
}

export interface SocialMetrics {
  symbol: string
  platform: 'twitter' | 'reddit' | 'telegram' | 'discord'
  followers: number
  mentions24h: number
  sentiment: number
  engagement: number
  influencerScore: number
  trendingScore: number
}

export interface DeFiProtocol {
  id: string
  name: string
  symbol: string
  logo: string
  tvl: number
  tvlChange24h: number
  tvlChange7d: number
  category: string
  chains: string[]
  apy?: number
  volume24h?: number
  users?: number
  description?: string
  website?: string
  twitter?: string
}

export interface LiquidityPool {
  id: string
  name: string
  protocol: string
  tokens: Array<{
    symbol: string
    address: string
    balance: number
    weight: number
  }>
  tvl: number
  volume24h: number
  volume7d: number
  fees24h: number
  apy: number
  apyBreakdown: Array<{
    type: 'trading' | 'rewards' | 'farming'
    apy: number
    token?: string
  }>
  network: string
  address: string
}

export interface GasTracker {
  network: string
  prices: {
    slow: number
    standard: number
    fast: number
    instant: number
  }
  estimatedTimes: {
    slow: number // seconds
    standard: number
    fast: number
    instant: number
  }
  lastUpdated: string
}

export interface ExchangeInfo {
  id: string
  name: string
  image: string
  trustScore: number
  trustScoreRank: number
  tradeVolume24h: number
  yearEstablished?: number
  country?: string
  description?: string
  url?: string
  hasTradeApi: boolean
  centralizedCoins: number
  markets: number
  fiatSupported: string[]
  cryptoSupported: string[]
}

class MarketService {
  private readonly basePath = '/market'

  /**
   * Get market assets with pagination and filtering
   */
  async getAssets(params?: {
    vs_currency?: string
    ids?: string[]
    category?: string
    order?: 'market_cap_desc' | 'market_cap_asc' | 'volume_desc' | 'volume_asc' | 'id_asc' | 'id_desc'
    per_page?: number
    page?: number
    sparkline?: boolean
    price_change_percentage?: string
  }): Promise<ApiResponse<PaginatedResponse<Asset>>> {
    return apiClient.getPaginated(`${this.basePath}/assets`, {
      vs_currency: 'usd',
      order: 'market_cap_desc',
      per_page: 50,
      page: 1,
      sparkline: true,
      price_change_percentage: '1h,24h,7d,30d',
      ...params
    })
  }

  /**
   * Get specific asset details
   */
  async getAsset(id: string): Promise<ApiResponse<Asset & {
    description: string
    links: Record<string, string>
    market_data: Record<string, any>
    community_data: Record<string, any>
    developer_data: Record<string, any>
  }>> {
    return apiClient.get(`${this.basePath}/assets/${id}`)
  }

  /**
   * Search assets by name or symbol
   */
  async searchAssets(query: string): Promise<ApiResponse<Array<{
    id: string
    symbol: string
    name: string
    image: string
    market_cap_rank: number
  }>>> {
    return apiClient.get(`${this.basePath}/search`, {
      params: { q: query }
    })
  }

  /**
   * Get current price for multiple assets
   */
  async getPrices(symbols: string[], vs_currency: string = 'usd'): Promise<ApiResponse<Record<string, PriceData>>> {
    return apiClient.get(`${this.basePath}/prices`, {
      params: {
        symbols: symbols.join(','),
        vs_currency
      }
    })
  }

  /**
   * Get historical price chart data
   */
  async getChartData(
    symbol: string,
    timeframe: ChartData['timeframe'],
    from?: number,
    to?: number
  ): Promise<ApiResponse<ChartData>> {
    return apiClient.get(`${this.basePath}/chart/${symbol}`, {
      params: {
        timeframe,
        from,
        to
      }
    })
  }

  /**
   * Get global market statistics
   */
  async getMarketStats(): Promise<ApiResponse<MarketStats>> {
    return apiClient.get(`${this.basePath}/stats`)
  }

  /**
   * Get trending assets
   */
  async getTrendingAssets(): Promise<ApiResponse<{
    trending: TrendingAsset[]
    top_gainers: TrendingAsset[]
    top_losers: TrendingAsset[]
    most_searched: TrendingAsset[]
  }>> {
    return apiClient.get(`${this.basePath}/trending`)
  }

  /**
   * Get whale transactions
   */
  async getWhaleTransactions(params?: {
    symbol?: string
    min_value?: number
    type?: WhaleTransaction['type']
    limit?: number
    offset?: number
  }): Promise<ApiResponse<PaginatedResponse<WhaleTransaction>>> {
    return apiClient.getPaginated(`${this.basePath}/whale-transactions`, params)
  }

  /**
   * Get market sentiment data
   */
  async getMarketSentiment(): Promise<ApiResponse<MarketSentiment>> {
    return apiClient.get(`${this.basePath}/sentiment`)
  }

  /**
   * Get news articles
   */
  async getNews(params?: {
    symbols?: string[]
    categories?: string[]
    sentiment?: 'positive' | 'neutral' | 'negative'
    limit?: number
    offset?: number
  }): Promise<ApiResponse<PaginatedResponse<NewsArticle>>> {
    return apiClient.getPaginated(`${this.basePath}/news`, params)
  }

  /**
   * Get social metrics for assets
   */
  async getSocialMetrics(symbols: string[]): Promise<ApiResponse<Record<string, SocialMetrics[]>>> {
    return apiClient.get(`${this.basePath}/social`, {
      params: {
        symbols: symbols.join(',')
      }
    })
  }

  // DeFi APIs

  /**
   * Get DeFi protocols
   */
  async getDeFiProtocols(params?: {
    category?: string
    chain?: string
    sort?: 'tvl' | 'name' | 'change_24h' | 'change_7d'
    order?: 'desc' | 'asc'
    limit?: number
  }): Promise<ApiResponse<DeFiProtocol[]>> {
    return apiClient.get(`${this.basePath}/defi/protocols`, {
      params
    })
  }

  /**
   * Get specific DeFi protocol details
   */
  async getDeFiProtocol(id: string): Promise<ApiResponse<DeFiProtocol & {
    chains: Array<{
      chain: string
      tvl: number
      tokenBreakdowns: Record<string, number>
    }>
    tokensInUsd: Record<string, number>
    chartData: Array<{
      date: string
      tvl: number
    }>
  }>> {
    return apiClient.get(`${this.basePath}/defi/protocols/${id}`)
  }

  /**
   * Get liquidity pools
   */
  async getLiquidityPools(params?: {
    protocol?: string
    network?: string
    tokens?: string[]
    min_tvl?: number
    min_apy?: number
    sort?: 'tvl' | 'apy' | 'volume' | 'fees'
    limit?: number
  }): Promise<ApiResponse<LiquidityPool[]>> {
    return apiClient.get(`${this.basePath}/defi/pools`, {
      params
    })
  }

  /**
   * Get gas prices for different networks
   */
  async getGasPrices(networks?: string[]): Promise<ApiResponse<Record<string, GasTracker>>> {
    return apiClient.get(`${this.basePath}/gas`, {
      params: {
        networks: networks?.join(',')
      }
    })
  }

  /**
   * Get exchange information
   */
  async getExchanges(params?: {
    per_page?: number
    page?: number
  }): Promise<ApiResponse<PaginatedResponse<ExchangeInfo>>> {
    return apiClient.getPaginated(`${this.basePath}/exchanges`, params)
  }

  /**
   * Get specific exchange details
   */
  async getExchange(id: string): Promise<ApiResponse<ExchangeInfo & {
    tickers: Array<{
      base: string
      target: string
      market: string
      last: number
      volume: number
      converted_last: Record<string, number>
      converted_volume: Record<string, number>
      trust_score: string
      bid_ask_spread_percentage: number
      timestamp: string
      last_traded_at: string
      last_fetch_at: string
      is_anomaly: boolean
      is_stale: boolean
      trade_url: string
      token_info_url?: string
      coin_id: string
      target_coin_id?: string
    }>
  }>> {
    return apiClient.get(`${this.basePath}/exchanges/${id}`)
  }

  // Real-time Data Streams

  /**
   * Subscribe to real-time price updates
   */
  subscribeToPrices(
    symbols: string[],
    callback: (data: {
      symbol: string
      price: number
      change24h: number
      volume24h: number
      timestamp: string
    }) => void
  ): WebSocket {
    const ws = apiClient.createWebSocket(`/market/prices/subscribe`)

    ws.onopen = () => {
      ws.send(JSON.stringify({
        action: 'subscribe',
        symbols
      }))
    }

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data)
      if (data.type === 'price_update') {
        callback(data.payload)
      }
    }

    return ws
  }

  /**
   * Subscribe to whale transaction alerts
   */
  subscribeToWhaleTransactions(
    filters: {
      minValue?: number
      symbols?: string[]
      types?: WhaleTransaction['type'][]
    },
    callback: (transaction: WhaleTransaction) => void
  ): WebSocket {
    const ws = apiClient.createWebSocket(`/market/whale-transactions/subscribe`)

    ws.onopen = () => {
      ws.send(JSON.stringify({
        action: 'subscribe',
        filters
      }))
    }

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data)
      if (data.type === 'whale_transaction') {
        callback(data.payload)
      }
    }

    return ws
  }

  /**
   * Subscribe to market sentiment updates
   */
  subscribeToSentiment(
    callback: (sentiment: MarketSentiment) => void
  ): WebSocket {
    const ws = apiClient.createWebSocket(`/market/sentiment/subscribe`)

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data)
      if (data.type === 'sentiment_update') {
        callback(data.payload)
      }
    }

    return ws
  }

  /**
   * Get historical data export
   */
  async exportHistoricalData(params: {
    symbol: string
    startDate: string
    endDate: string
    interval: '1m' | '5m' | '15m' | '1h' | '4h' | '1d'
    format: 'csv' | 'json'
  }): Promise<ApiResponse<{
    downloadUrl: string
    expiresAt: string
  }>> {
    return apiClient.post(`${this.basePath}/export`, params)
  }

  /**
   * Get market alerts
   */
  async getMarketAlerts(): Promise<ApiResponse<Array<{
    id: string
    type: 'price' | 'volume' | 'market_cap' | 'news'
    symbol: string
    condition: string
    value: number
    active: boolean
    triggered: boolean
    createdAt: string
  }>>> {
    return apiClient.get(`${this.basePath}/alerts`)
  }

  /**
   * Create market alert
   */
  async createMarketAlert(data: {
    type: 'price' | 'volume' | 'market_cap' | 'news'
    symbol: string
    condition: 'above' | 'below' | 'crosses_above' | 'crosses_below'
    value: number
    notification: 'email' | 'push' | 'both'
  }): Promise<ApiResponse<void>> {
    return apiClient.post(`${this.basePath}/alerts`, data)
  }

  /**
   * Delete market alert
   */
  async deleteMarketAlert(alertId: string): Promise<ApiResponse<void>> {
    return apiClient.delete(`${this.basePath}/alerts/${alertId}`)
  }

  /**
   * Get market analysis
   */
  async getMarketAnalysis(symbol: string): Promise<ApiResponse<{
    technicalAnalysis: {
      trend: 'bullish' | 'bearish' | 'neutral'
      support: number[]
      resistance: number[]
      indicators: Record<string, {
        value: number
        signal: 'buy' | 'sell' | 'neutral'
      }>
    }
    fundamentalAnalysis: {
      score: number
      factors: Array<{
        factor: string
        score: number
        weight: number
        description: string
      }>
    }
    sentiment: {
      overall: number
      breakdown: Record<string, number>
      sources: Array<{
        source: string
        sentiment: number
        weight: number
      }>
    }
    priceTargets: Array<{
      timeframe: string
      target: number
      probability: number
      reasoning: string
    }>
  }>> {
    return apiClient.get(`${this.basePath}/analysis/${symbol}`)
  }
}

// Export singleton instance
export const marketService = new MarketService()

// Export class for testing
export { MarketService }