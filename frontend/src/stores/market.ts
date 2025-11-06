import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

import type { MarketData, TokenPrice, WhaleActivity } from '@/types'

export const useMarketStore = defineStore('market', () => {
  // State
  const isConnected = ref(false)
  const isLoading = ref(false)
  const lastUpdated = ref<string>('')
  const error = ref<string>('')

  // Market data
  const globalMarketCap = ref<number>(2800000000000) // $2.8T default
  const marketCapChange = ref<number>(1.25)
  const volume24h = ref<number>(85000000000) // $85B default
  const dominanceBTC = ref<number>(42.5)
  const dominanceETH = ref<number>(18.3)

  // Gas and network data
  const gasPrice = ref<number>(25) // gwei
  const gasPriceUSD = ref<number>(2.50)
  const blockNumber = ref<number>(18500000)

  // Whale activity
  const activeWhales = ref<number>(247)
  const whaleActivity = ref<WhaleActivity[]>([])
  const topMovers = ref<TokenPrice[]>([])

  // Fear & Greed Index
  const fearGreedIndex = ref<number>(65)
  const fearGreedLabel = ref<string>('Greed')

  // Trending tokens
  const trendingTokens = ref<TokenPrice[]>([])

  // Token prices cache
  const tokenPrices = ref<Map<string, TokenPrice>>(new Map())

  // Getters
  const marketStatus = computed(() => {
    if (!isConnected.value) return 'disconnected'
    if (isLoading.value) return 'loading'
    if (error.value) return 'error'
    return 'connected'
  })

  const formattedMarketCap = computed(() => {
    if (globalMarketCap.value >= 1e12) {
      return `$${(globalMarketCap.value / 1e12).toFixed(2)}T`
    }
    if (globalMarketCap.value >= 1e9) {
      return `$${(globalMarketCap.value / 1e9).toFixed(2)}B`
    }
    return `$${globalMarketCap.value.toLocaleString()}`
  })

  const formattedVolume = computed(() => {
    if (volume24h.value >= 1e9) {
      return `$${(volume24h.value / 1e9).toFixed(1)}B`
    }
    if (volume24h.value >= 1e6) {
      return `$${(volume24h.value / 1e6).toFixed(1)}M`
    }
    return `$${volume24h.value.toLocaleString()}`
  })

  const gasPriceStatus = computed(() => {
    if (gasPrice.value <= 20) return 'low'
    if (gasPrice.value <= 50) return 'medium'
    return 'high'
  })

  const recentWhaleActivity = computed(() =>
    whaleActivity.value.slice(0, 10)
  )

  // Actions
  async function fetchMarketData(): Promise<void> {
    isLoading.value = true
    error.value = ''

    try {
      // Simulate API call - replace with actual API calls
      await simulateMarketDataFetch()

      isConnected.value = true
      lastUpdated.value = new Date().toISOString()
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch market data'
      isConnected.value = false
    } finally {
      isLoading.value = false
    }
  }

  async function fetchTokenPrice(symbol: string): Promise<TokenPrice | null> {
    try {
      // Check cache first
      const cached = tokenPrices.value.get(symbol.toLowerCase())
      if (cached && isCacheValid(cached.lastUpdated)) {
        return cached
      }

      // Simulate API call
      const tokenPrice = await simulateTokenPriceFetch(symbol)

      // Update cache
      tokenPrices.value.set(symbol.toLowerCase(), tokenPrice)

      return tokenPrice
    } catch (err) {
      console.error(`Failed to fetch price for ${symbol}:`, err)
      return null
    }
  }

  async function fetchWhaleActivity(): Promise<void> {
    try {
      // Simulate whale activity fetch
      const activity = await simulateWhaleActivityFetch()
      whaleActivity.value = activity
    } catch (err) {
      console.error('Failed to fetch whale activity:', err)
    }
  }

  async function updateGasPrice(): Promise<void> {
    try {
      // Simulate gas price fetch
      const newGasPrice = await simulateGasPriceFetch()
      gasPrice.value = newGasPrice.standard
      gasPriceUSD.value = newGasPrice.usd
    } catch (err) {
      console.error('Failed to update gas price:', err)
    }
  }

  function addWhaleActivity(activity: WhaleActivity): void {
    whaleActivity.value.unshift(activity)
    // Keep only last 100 activities
    if (whaleActivity.value.length > 100) {
      whaleActivity.value = whaleActivity.value.slice(0, 100)
    }
  }

  function updateTokenPrice(symbol: string, price: TokenPrice): void {
    tokenPrices.value.set(symbol.toLowerCase(), price)
  }

  function getTokenPrice(symbol: string): TokenPrice | null {
    return tokenPrices.value.get(symbol.toLowerCase()) || null
  }

  function clearCache(): void {
    tokenPrices.value.clear()
  }

  // Simulation functions (replace with real API calls)
  async function simulateMarketDataFetch(): Promise<void> {
    // Simulate network delay
    await new Promise(resolve => setTimeout(resolve, 1000))

    // Simulate some market movements
    const change = (Math.random() - 0.5) * 5 // Random change between -2.5% and +2.5%
    marketCapChange.value = change
    globalMarketCap.value = globalMarketCap.value * (1 + change / 100)

    // Update volume with some randomness
    volume24h.value = 85000000000 + (Math.random() - 0.5) * 20000000000

    // Update gas price
    gasPrice.value = Math.max(5, Math.min(200, gasPrice.value + (Math.random() - 0.5) * 10))

    // Update whale count
    activeWhales.value = Math.floor(200 + Math.random() * 100)

    // Update fear & greed
    fearGreedIndex.value = Math.floor(Math.random() * 100)
    if (fearGreedIndex.value <= 25) fearGreedLabel.value = 'Extreme Fear'
    else if (fearGreedIndex.value <= 45) fearGreedLabel.value = 'Fear'
    else if (fearGreedIndex.value <= 55) fearGreedLabel.value = 'Neutral'
    else if (fearGreedIndex.value <= 75) fearGreedLabel.value = 'Greed'
    else fearGreedLabel.value = 'Extreme Greed'
  }

  async function simulateTokenPriceFetch(symbol: string): Promise<TokenPrice> {
    await new Promise(resolve => setTimeout(resolve, 500))

    // Mock token prices
    const mockPrices: Record<string, number> = {
      eth: 3200 + (Math.random() - 0.5) * 200,
      btc: 65000 + (Math.random() - 0.5) * 5000,
      usdc: 1.0,
      usdt: 1.0,
      dai: 1.0,
      matic: 0.85 + (Math.random() - 0.5) * 0.2,
      arb: 1.2 + (Math.random() - 0.5) * 0.3,
      op: 2.5 + (Math.random() - 0.5) * 0.5,
    }

    const basePrice = mockPrices[symbol.toLowerCase()] || 100
    const change24h = (Math.random() - 0.5) * 20 // -10% to +10%

    return {
      symbol: symbol.toUpperCase(),
      name: getTokenName(symbol),
      price: basePrice,
      change24h,
      volume24h: Math.random() * 1000000000,
      marketCap: basePrice * Math.random() * 1000000000,
      lastUpdated: new Date().toISOString(),
    }
  }

  async function simulateWhaleActivityFetch(): Promise<WhaleActivity[]> {
    await new Promise(resolve => setTimeout(resolve, 300))

    const activities: WhaleActivity[] = []
    const tokens = ['ETH', 'BTC', 'USDC', 'USDT', 'MATIC', 'ARB']
    const types = ['buy', 'sell', 'transfer'] as const

    for (let i = 0; i < 20; i++) {
      const token = tokens[Math.floor(Math.random() * tokens.length)]
      const type = types[Math.floor(Math.random() * types.length)]
      const amount = Math.floor(Math.random() * 10000) + 1000

      activities.push({
        id: `whale-${Date.now()}-${i}`,
        address: `0x${Math.random().toString(16).substr(2, 40)}`,
        type,
        token,
        amount: amount.toString(),
        usdValue: amount * (Math.random() * 10 + 1),
        timestamp: new Date(Date.now() - Math.random() * 3600000).toISOString(), // Last hour
        txHash: `0x${Math.random().toString(16).substr(2, 64)}`,
        blockNumber: Math.floor(Math.random() * 1000) + 18500000,
      })
    }

    return activities.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
  }

  async function simulateGasPriceFetch(): Promise<{ standard: number; fast: number; usd: number }> {
    await new Promise(resolve => setTimeout(resolve, 200))

    const standard = Math.max(5, Math.min(200, gasPrice.value + (Math.random() - 0.5) * 5))
    return {
      standard,
      fast: standard * 1.2,
      usd: standard * 0.1,
    }
  }

  function getTokenName(symbol: string): string {
    const names: Record<string, string> = {
      eth: 'Ethereum',
      btc: 'Bitcoin',
      usdc: 'USD Coin',
      usdt: 'Tether',
      dai: 'Dai',
      matic: 'Polygon',
      arb: 'Arbitrum',
      op: 'Optimism',
    }
    return names[symbol.toLowerCase()] || symbol.toUpperCase()
  }

  function isCacheValid(lastUpdated: string): boolean {
    const cacheTime = 30000 // 30 seconds
    return Date.now() - new Date(lastUpdated).getTime() < cacheTime
  }

  // Initialize market data on store creation
  function initialize(): void {
    fetchMarketData()

    // Set up periodic updates
    setInterval(fetchMarketData, 30000) // Update every 30 seconds
    setInterval(updateGasPrice, 15000) // Update gas price every 15 seconds
    setInterval(fetchWhaleActivity, 60000) // Update whale activity every minute
  }

  return {
    // State
    isConnected,
    isLoading,
    lastUpdated,
    error,
    globalMarketCap,
    marketCapChange,
    volume24h,
    dominanceBTC,
    dominanceETH,
    gasPrice,
    gasPriceUSD,
    blockNumber,
    activeWhales,
    whaleActivity,
    topMovers,
    fearGreedIndex,
    fearGreedLabel,
    trendingTokens,

    // Getters
    marketStatus,
    formattedMarketCap,
    formattedVolume,
    gasPriceStatus,
    recentWhaleActivity,

    // Actions
    fetchMarketData,
    fetchTokenPrice,
    fetchWhaleActivity,
    updateGasPrice,
    addWhaleActivity,
    updateTokenPrice,
    getTokenPrice,
    clearCache,
    initialize,
  }
})