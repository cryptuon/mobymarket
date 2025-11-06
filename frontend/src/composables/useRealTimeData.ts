import { ref, computed, onMounted, onUnmounted } from 'vue'
import { storeToRefs } from 'pinia'

import { globalWebSocket } from '@/services/websocket'
import { useMarketStore } from '@/stores/market'
import { useNotificationStore } from '@/stores/notifications'
import type {
  WhaleActivity,
  TokenPrice,
  MarketData,
  ConnectionState
} from '@/types'

export function useRealTimeData() {
  const marketStore = useMarketStore()
  const notificationStore = useNotificationStore()

  const connectionState = ref<ConnectionState>('disconnected')
  const lastUpdate = ref<string>('')
  const subscriptions = ref<string[]>([])

  // Real-time data streams
  const liveWhaleActivity = ref<WhaleActivity[]>([])
  const livePrices = ref<Map<string, TokenPrice>>(new Map())
  const liveMarketData = ref<MarketData | null>(null)

  // Connection status
  const isConnected = computed(() => connectionState.value === 'connected')
  const isConnecting = computed(() => connectionState.value === 'connecting')
  const hasError = computed(() => connectionState.value === 'error')

  // Data freshness
  const isDataFresh = computed(() => {
    if (!lastUpdate.value) return false
    return Date.now() - new Date(lastUpdate.value).getTime() < 60000 // 1 minute
  })

  // Initialize connection
  async function connect() {
    try {
      await globalWebSocket.connect()
      setupSubscriptions()
    } catch (error) {
      console.error('Failed to connect to real-time data:', error)
      notificationStore.notifySystem(
        'Connection Failed',
        'Unable to connect to real-time data feed',
        'error'
      )
    }
  }

  // Setup data subscriptions
  function setupSubscriptions() {
    // Subscribe to whale activity
    const whaleActivitySub = globalWebSocket.subscribe('whale-activity', handleWhaleActivity)
    subscriptions.value.push(whaleActivitySub)

    // Subscribe to price updates
    const priceUpdateSub = globalWebSocket.subscribe('price-updates', handlePriceUpdate)
    subscriptions.value.push(priceUpdateSub)

    // Subscribe to market data
    const marketDataSub = globalWebSocket.subscribe('market-data', handleMarketData)
    subscriptions.value.push(marketDataSub)

    // Subscribe to gas price updates
    const gasPriceSub = globalWebSocket.subscribe('gas-prices', handleGasPrice)
    subscriptions.value.push(gasPriceSub)

    console.log('Real-time data subscriptions established')
  }

  // Data handlers
  function handleWhaleActivity(data: WhaleActivity) {
    // Add to live activity feed
    liveWhaleActivity.value.unshift(data)

    // Keep only last 50 activities
    if (liveWhaleActivity.value.length > 50) {
      liveWhaleActivity.value = liveWhaleActivity.value.slice(0, 50)
    }

    // Update market store
    marketStore.addWhaleActivity(data)

    // Show notification for significant whale activity
    if (data.usdValue > 1000000) { // $1M+ trades
      notificationStore.notifyWhaleActivity({
        amount: data.amount,
        token: data.token,
        type: data.type,
        address: data.address,
        txHash: data.txHash
      })
    }

    lastUpdate.value = new Date().toISOString()
  }

  function handlePriceUpdate(data: TokenPrice) {
    // Update live prices
    livePrices.value.set(data.symbol.toLowerCase(), data)

    // Update market store
    marketStore.updateTokenPrice(data.symbol, data)

    lastUpdate.value = new Date().toISOString()
  }

  function handleMarketData(data: MarketData) {
    liveMarketData.value = data
    lastUpdate.value = new Date().toISOString()
  }

  function handleGasPrice(data: { standard: number; fast: number; usd: number }) {
    marketStore.gasPrice = data.standard
    marketStore.gasPriceUSD = data.usd
    lastUpdate.value = new Date().toISOString()
  }

  // Subscribe to specific token prices
  function subscribeToToken(symbol: string): string {
    const subscriptionId = globalWebSocket.subscribe(`price-${symbol.toLowerCase()}`, (data) => {
      handlePriceUpdate(data)
    })

    subscriptions.value.push(subscriptionId)
    return subscriptionId
  }

  // Subscribe to specific trading pair
  function subscribeToTradingPair(tokenA: string, tokenB: string): string {
    const pairKey = `${tokenA.toLowerCase()}-${tokenB.toLowerCase()}`
    const subscriptionId = globalWebSocket.subscribe(`pair-${pairKey}`, (data) => {
      // Handle trading pair specific data like volume, liquidity, etc.
      console.log('Trading pair update:', data)
    })

    subscriptions.value.push(subscriptionId)
    return subscriptionId
  }

  // Get live price for token
  function getLivePrice(symbol: string): TokenPrice | null {
    return livePrices.value.get(symbol.toLowerCase()) || null
  }

  // Get recent whale activity for token
  function getWhaleActivityForToken(token: string): WhaleActivity[] {
    return liveWhaleActivity.value.filter(activity =>
      activity.token.toLowerCase() === token.toLowerCase()
    )
  }

  // Send custom subscription
  function sendSubscription(channel: string, params?: any): string {
    return globalWebSocket.subscribe(channel, (data) => {
      console.log(`Received data for ${channel}:`, data)
    })
  }

  // Cleanup subscriptions
  function cleanup() {
    subscriptions.value.forEach(subId => {
      globalWebSocket.unsubscribe(subId)
    })
    subscriptions.value = []
  }

  // Disconnect
  function disconnect() {
    cleanup()
    globalWebSocket.disconnect()
  }

  // Simulated real-time data (for development)
  function startSimulatedData() {
    const interval = setInterval(() => {
      if (!isConnected.value) return

      // Simulate whale activity
      if (Math.random() < 0.3) { // 30% chance
        const tokens = ['ETH', 'BTC', 'USDC', 'USDT']
        const types: ('buy' | 'sell' | 'transfer')[] = ['buy', 'sell', 'transfer']

        const simulatedActivity: WhaleActivity = {
          id: `sim-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
          address: `0x${Math.random().toString(16).substr(2, 40)}`,
          type: types[Math.floor(Math.random() * types.length)],
          token: tokens[Math.floor(Math.random() * tokens.length)],
          amount: (Math.random() * 1000 + 100).toFixed(2),
          usdValue: Math.random() * 5000000 + 100000,
          timestamp: new Date().toISOString(),
          txHash: `0x${Math.random().toString(16).substr(2, 64)}`,
          blockNumber: Math.floor(Math.random() * 1000) + 18500000
        }

        handleWhaleActivity(simulatedActivity)
      }

      // Simulate price updates
      if (Math.random() < 0.5) { // 50% chance
        const tokens = ['ETH', 'BTC', 'USDC']
        const token = tokens[Math.floor(Math.random() * tokens.length)]

        const basePrice = token === 'ETH' ? 3200 : token === 'BTC' ? 65000 : 1
        const change = (Math.random() - 0.5) * 0.02 // ±1% change

        const simulatedPrice: TokenPrice = {
          symbol: token,
          name: token === 'ETH' ? 'Ethereum' : token === 'BTC' ? 'Bitcoin' : 'USD Coin',
          price: basePrice * (1 + change),
          change24h: change * 100,
          volume24h: Math.random() * 1000000000,
          marketCap: basePrice * 120000000,
          lastUpdated: new Date().toISOString()
        }

        handlePriceUpdate(simulatedPrice)
      }
    }, 2000) // Every 2 seconds

    return () => clearInterval(interval)
  }

  // Setup WebSocket event handlers
  onMounted(() => {
    globalWebSocket.onStateChanged((state) => {
      connectionState.value = state
    })

    globalWebSocket.onErrorOccurred((error) => {
      console.error('WebSocket error:', error)
      notificationStore.notifySystem(
        'Connection Error',
        'Real-time data connection encountered an error',
        'error'
      )
    })

    // Auto-connect on mount
    connect()

    // Start simulated data in development
    if (import.meta.env.DEV) {
      const stopSimulation = startSimulatedData()
      onUnmounted(stopSimulation)
    }
  })

  onUnmounted(() => {
    cleanup()
  })

  return {
    // Connection state
    connectionState,
    isConnected,
    isConnecting,
    hasError,
    lastUpdate,
    isDataFresh,

    // Live data
    liveWhaleActivity,
    livePrices,
    liveMarketData,

    // Methods
    connect,
    disconnect,
    subscribeToToken,
    subscribeToTradingPair,
    sendSubscription,
    getLivePrice,
    getWhaleActivityForToken,
    cleanup,

    // For development
    startSimulatedData
  }
}

// Global real-time data instance
export const useGlobalRealTimeData = () => {
  return useRealTimeData()
}