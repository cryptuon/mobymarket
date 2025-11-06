import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

import type {
  Quote,
  TradeRequest,
  TradeResult,
  TradingPair,
  TokenInfo,
  TradingState
} from '@/types'

export const useTradingStore = defineStore('trading', () => {
  // State
  const isLoading = ref(false)
  const activeQuotes = ref<Map<string, Quote>>(new Map())
  const activeTrades = ref<TradeResult[]>([])
  const tradingHistory = ref<TradeResult[]>([])
  const favoritePairs = ref<TradingPair[]>([])

  // Trading preferences
  const defaultSlippage = ref<number>(0.5) // 0.5%
  const defaultDeadline = ref<number>(20) // 20 minutes
  const usePrivacy = ref<boolean>(false)
  const mevProtection = ref<boolean>(true)
  const autoRefresh = ref<boolean>(true)
  const soundEnabled = ref<boolean>(true)

  // Current trade state
  const tokenIn = ref<string>('')
  const tokenOut = ref<string>('')
  const amountIn = ref<string>('')
  const amountOut = ref<string>('')
  const selectedQuote = ref<Quote | null>(null)
  const priceImpact = ref<number>(0)
  const gasEstimate = ref<string>('0')

  // Popular tokens for quick selection
  const popularTokens = ref<TokenInfo[]>([
    {
      address: '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2',
      symbol: 'WETH',
      name: 'Wrapped Ether',
      decimals: 18,
      chainId: 1,
      logoURI: '/tokens/weth.svg'
    },
    {
      address: '0xA0b86a33E6847d7b1e7bCd15F6FdE5d2b9FC1234',
      symbol: 'USDC',
      name: 'USD Coin',
      decimals: 6,
      chainId: 1,
      logoURI: '/tokens/usdc.svg'
    },
    {
      address: '0xdAC17F958D2ee523a2206206994597C13D831ec7',
      symbol: 'USDT',
      name: 'Tether USD',
      decimals: 6,
      chainId: 1,
      logoURI: '/tokens/usdt.svg'
    },
    {
      address: '0x6B175474E89094C44Da98b954EedeAC495271d0F',
      symbol: 'DAI',
      name: 'Dai Stablecoin',
      decimals: 18,
      chainId: 1,
      logoURI: '/tokens/dai.svg'
    }
  ])

  // Getters
  const currentQuote = computed(() => {
    if (!tokenIn.value || !tokenOut.value || !amountIn.value) return null
    const quoteKey = `${tokenIn.value}-${tokenOut.value}-${amountIn.value}`
    return activeQuotes.value.get(quoteKey) || null
  })

  const isValidTrade = computed(() => {
    return !!(
      tokenIn.value &&
      tokenOut.value &&
      amountIn.value &&
      parseFloat(amountIn.value) > 0 &&
      currentQuote.value
    )
  })

  const tradingState = computed<TradingState>(() => ({
    quotes: Object.fromEntries(activeQuotes.value),
    activeTrades: activeTrades.value,
    tradingHistory: tradingHistory.value,
    favorites: favoritePairs.value,
    defaultSlippage: defaultSlippage.value,
    defaultDeadline: defaultDeadline.value,
    preferences: {
      usePrivacy: usePrivacy.value,
      mevProtection: mevProtection.value,
      autoRefresh: autoRefresh.value,
      soundEnabled: soundEnabled.value
    }
  }))

  const pendingTrades = computed(() =>
    activeTrades.value.filter(trade => trade.status === 'pending')
  )

  const completedTrades = computed(() =>
    tradingHistory.value.filter(trade => trade.status === 'confirmed')
  )

  const failedTrades = computed(() =>
    tradingHistory.value.filter(trade => trade.status === 'failed')
  )

  // Actions
  async function fetchQuote(tokenInAddress: string, tokenOutAddress: string, amount: string): Promise<Quote | null> {
    if (!tokenInAddress || !tokenOutAddress || !amount || parseFloat(amount) <= 0) {
      return null
    }

    isLoading.value = true

    try {
      // Simulate API call to get quote
      const quote = await simulateQuoteFetch(tokenInAddress, tokenOutAddress, amount)

      const quoteKey = `${tokenInAddress}-${tokenOutAddress}-${amount}`
      activeQuotes.value.set(quoteKey, quote)

      // Update reactive state
      if (tokenInAddress === tokenIn.value && tokenOutAddress === tokenOut.value) {
        amountOut.value = quote.amountOut
        priceImpact.value = quote.priceImpact
        gasEstimate.value = quote.gasEstimate
      }

      return quote
    } catch (error) {
      console.error('Failed to fetch quote:', error)
      return null
    } finally {
      isLoading.value = false
    }
  }

  async function executeTrade(tradeRequest: TradeRequest): Promise<TradeResult> {
    if (!isValidTrade.value) {
      throw new Error('Invalid trade parameters')
    }

    try {
      // Simulate trade execution
      const tradeResult = await simulateTradeExecution(tradeRequest)

      // Add to active trades
      activeTrades.value.unshift(tradeResult)

      // Clear current trade state
      clearTradeState()

      return tradeResult
    } catch (error) {
      throw error
    }
  }

  function updateTradeStatus(tradeId: string, status: TradeResult['status'], hash?: string) {
    const tradeIndex = activeTrades.value.findIndex(trade => trade.id === tradeId)

    if (tradeIndex !== -1) {
      const trade = activeTrades.value[tradeIndex]
      trade.status = status

      if (hash) {
        trade.hash = hash
      }

      // Move completed/failed trades to history
      if (status === 'confirmed' || status === 'failed') {
        const completedTrade = activeTrades.value.splice(tradeIndex, 1)[0]
        tradingHistory.value.unshift(completedTrade)
      }
    }
  }

  function setTokenIn(token: string) {
    if (token !== tokenOut.value) {
      tokenIn.value = token
      // Clear amounts when changing tokens
      amountIn.value = ''
      amountOut.value = ''
    }
  }

  function setTokenOut(token: string) {
    if (token !== tokenIn.value) {
      tokenOut.value = token
      // Clear amounts when changing tokens
      amountIn.value = ''
      amountOut.value = ''
    }
  }

  function swapTokens() {
    const tempToken = tokenIn.value
    tokenIn.value = tokenOut.value
    tokenOut.value = tempToken

    // Swap amounts too
    const tempAmount = amountIn.value
    amountIn.value = amountOut.value
    amountOut.value = tempAmount
  }

  function setAmountIn(amount: string) {
    amountIn.value = amount
    amountOut.value = '' // Clear output amount for re-calculation
  }

  function clearTradeState() {
    tokenIn.value = ''
    tokenOut.value = ''
    amountIn.value = ''
    amountOut.value = ''
    selectedQuote.value = null
    priceImpact.value = 0
    gasEstimate.value = '0'
  }

  function addToFavorites(pair: TradingPair) {
    const exists = favoritePairs.value.some(
      fav => fav.tokenIn.address === pair.tokenIn.address &&
             fav.tokenOut.address === pair.tokenOut.address
    )

    if (!exists) {
      favoritePairs.value.push(pair)
    }
  }

  function removeFromFavorites(pair: TradingPair) {
    const index = favoritePairs.value.findIndex(
      fav => fav.tokenIn.address === pair.tokenIn.address &&
             fav.tokenOut.address === pair.tokenOut.address
    )

    if (index !== -1) {
      favoritePairs.value.splice(index, 1)
    }
  }

  function updatePreferences(updates: Partial<TradingState['preferences']>) {
    if (updates.usePrivacy !== undefined) usePrivacy.value = updates.usePrivacy
    if (updates.mevProtection !== undefined) mevProtection.value = updates.mevProtection
    if (updates.autoRefresh !== undefined) autoRefresh.value = updates.autoRefresh
    if (updates.soundEnabled !== undefined) soundEnabled.value = updates.soundEnabled
  }

  function setSlippage(slippage: number) {
    defaultSlippage.value = Math.max(0.1, Math.min(50, slippage)) // Clamp between 0.1% and 50%
  }

  function setDeadline(deadline: number) {
    defaultDeadline.value = Math.max(1, Math.min(180, deadline)) // Clamp between 1 and 180 minutes
  }

  // Simulation functions (replace with real API calls)
  async function simulateQuoteFetch(
    tokenIn: string,
    tokenOut: string,
    amount: string
  ): Promise<Quote> {
    // Simulate network delay
    await new Promise(resolve => setTimeout(resolve, 800))

    const amountNum = parseFloat(amount)
    const randomRate = 0.95 + Math.random() * 0.1 // Random rate between 0.95-1.05
    const outputAmount = (amountNum * randomRate).toFixed(6)

    const priceImpact = Math.random() * 2 // Random price impact 0-2%
    const gasEstimate = (21000 + Math.random() * 150000).toFixed(0) // Random gas estimate

    return {
      id: `quote-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      tokenIn,
      tokenOut,
      amountIn: amount,
      amountOut: outputAmount,
      priceImpact,
      gasEstimate,
      route: [
        {
          dex: 'Uniswap V3',
          percentage: 60,
          amountIn: (amountNum * 0.6).toFixed(6),
          amountOut: (parseFloat(outputAmount) * 0.6).toFixed(6),
          priceImpact: priceImpact * 0.6,
          gasEstimate: (parseFloat(gasEstimate) * 0.6).toFixed(0)
        },
        {
          dex: '1inch',
          percentage: 40,
          amountIn: (amountNum * 0.4).toFixed(6),
          amountOut: (parseFloat(outputAmount) * 0.4).toFixed(6),
          priceImpact: priceImpact * 0.4,
          gasEstimate: (parseFloat(gasEstimate) * 0.4).toFixed(0)
        }
      ],
      validUntil: Date.now() + 30000, // Valid for 30 seconds
      timestamp: Date.now()
    }
  }

  async function simulateTradeExecution(tradeRequest: TradeRequest): Promise<TradeResult> {
    // Simulate network delay
    await new Promise(resolve => setTimeout(resolve, 1000))

    const tradeId = `trade-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
    const txHash = `0x${Math.random().toString(16).substr(2, 64)}`

    return {
      id: tradeId,
      hash: txHash,
      status: 'pending',
      amountIn: tradeRequest.amountIn,
      amountOut: amountOut.value,
      actualSlippage: Math.random() * defaultSlippage.value,
      gasUsed: gasEstimate.value,
      gasPrice: '20000000000', // 20 gwei
      timestamp: Date.now(),
      route: currentQuote.value?.route.map(r => ({
        dex: r.dex,
        amountIn: r.amountIn,
        amountOut: r.amountOut,
        gasUsed: r.gasEstimate
      })) || []
    }
  }

  // Auto-refresh quotes
  function startAutoRefresh() {
    if (!autoRefresh.value) return

    setInterval(() => {
      if (tokenIn.value && tokenOut.value && amountIn.value) {
        fetchQuote(tokenIn.value, tokenOut.value, amountIn.value)
      }
    }, 10000) // Refresh every 10 seconds
  }

  // Initialize default tokens for quick access
  function initializeDefaultTokens() {
    // Set default token pair (WETH -> USDC)
    if (popularTokens.value.length >= 2) {
      tokenIn.value = popularTokens.value[0].address
      tokenOut.value = popularTokens.value[1].address
    }
  }

  return {
    // State
    isLoading,
    activeQuotes,
    activeTrades,
    tradingHistory,
    favoritePairs,
    defaultSlippage,
    defaultDeadline,
    usePrivacy,
    mevProtection,
    autoRefresh,
    soundEnabled,
    tokenIn,
    tokenOut,
    amountIn,
    amountOut,
    selectedQuote,
    priceImpact,
    gasEstimate,
    popularTokens,

    // Getters
    currentQuote,
    isValidTrade,
    tradingState,
    pendingTrades,
    completedTrades,
    failedTrades,

    // Actions
    fetchQuote,
    executeTrade,
    updateTradeStatus,
    setTokenIn,
    setTokenOut,
    swapTokens,
    setAmountIn,
    clearTradeState,
    addToFavorites,
    removeFromFavorites,
    updatePreferences,
    setSlippage,
    setDeadline,
    startAutoRefresh,
    initializeDefaultTokens
  }
})