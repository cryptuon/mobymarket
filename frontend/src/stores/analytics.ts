import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export interface PortfolioMetrics {
  totalValue: number
  totalPnL: number
  dayPnL: number
  totalChange: number
  winRate: number
  sharpeRatio: number
  maxDrawdown: number
  volatility: number
  beta: number
  activePositions: number
}

export interface Position {
  id: string
  symbol: string
  name: string
  quantity: number
  avgPrice: number
  currentPrice: number
  value: number
  pnl: number
  pnlPercent: number
  allocation: number
  dayChange: number
  chain: string
  protocol?: string
  type: 'spot' | 'lp' | 'staked' | 'farming'
}

export interface Trade {
  id: string
  timestamp: string
  type: 'buy' | 'sell'
  symbol: string
  quantity: number
  price: number
  value: number
  fee: number
  pnl?: number
  status: 'pending' | 'completed' | 'failed'
  txHash?: string
  chain: string
}

export interface PerformanceData {
  timestamp: string
  portfolioValue: number
  pnl: number
  returns: number
  benchmark?: number
  drawdown: number
  volatility: number
}

export interface RiskMetrics {
  var95: number
  var99: number
  expectedShortfall: number
  concentration: number
  diversificationRatio: number
  correlationRisk: number
  liquidityRisk: number
  marketRisk: number
}

export interface YieldPosition {
  id: string
  protocol: string
  asset: string
  pool?: string
  staked: number
  value: number
  rewards: number
  apr: number
  apy: number
  lockupPeriod?: number
  maturityDate?: string
  autoCompound: boolean
  chain: string
}

export const useAnalyticsStore = defineStore('analytics', () => {
  // State
  const isLoading = ref(false)
  const lastUpdate = ref<string>('')

  // Portfolio data
  const portfolioMetrics = ref<PortfolioMetrics>({
    totalValue: 1000000,
    totalPnL: 125000,
    dayPnL: 12500,
    totalChange: 2.3,
    winRate: 73.8,
    sharpeRatio: 1.85,
    maxDrawdown: 8.5,
    volatility: 28.5,
    beta: 1.35,
    activePositions: 8
  })

  const positions = ref<Position[]>([
    {
      id: '1',
      symbol: 'ETH',
      name: 'Ethereum',
      quantity: 140.5,
      avgPrice: 3200,
      currentPrice: 3280,
      value: 460840,
      pnl: 11240,
      pnlPercent: 2.5,
      allocation: 46.1,
      dayChange: 1.8,
      chain: 'ethereum',
      type: 'spot'
    },
    {
      id: '2',
      symbol: 'BTC',
      name: 'Bitcoin',
      quantity: 4.2,
      avgPrice: 67000,
      currentPrice: 68500,
      value: 287700,
      pnl: 6300,
      pnlPercent: 2.2,
      allocation: 28.8,
      dayChange: 0.9,
      chain: 'bitcoin',
      type: 'spot'
    },
    {
      id: '3',
      symbol: 'USDC',
      name: 'USD Coin',
      quantity: 120000,
      avgPrice: 1.00,
      currentPrice: 1.001,
      value: 120120,
      pnl: 120,
      pnlPercent: 0.1,
      allocation: 12.0,
      dayChange: 0.1,
      chain: 'ethereum',
      type: 'spot'
    },
    {
      id: '4',
      symbol: 'ETH-USDC LP',
      name: 'Uniswap V3 ETH-USDC',
      quantity: 2.5,
      avgPrice: 45000,
      currentPrice: 46500,
      value: 116250,
      pnl: 3750,
      pnlPercent: 3.3,
      allocation: 11.6,
      dayChange: 2.1,
      chain: 'ethereum',
      protocol: 'Uniswap V3',
      type: 'lp'
    }
  ])

  const trades = ref<Trade[]>([
    {
      id: '1',
      timestamp: new Date(Date.now() - 3600000).toISOString(),
      type: 'buy',
      symbol: 'ETH',
      quantity: 5.0,
      price: 3250,
      value: 16250,
      fee: 25,
      status: 'completed',
      txHash: '0x1234...5678',
      chain: 'ethereum'
    },
    {
      id: '2',
      timestamp: new Date(Date.now() - 7200000).toISOString(),
      type: 'sell',
      symbol: 'BTC',
      quantity: 0.5,
      price: 68000,
      value: 34000,
      fee: 50,
      pnl: 2500,
      status: 'completed',
      txHash: '0x9abc...def0',
      chain: 'bitcoin'
    }
  ])

  const performanceHistory = ref<PerformanceData[]>([])

  const riskMetrics = ref<RiskMetrics>({
    var95: -45000,
    var99: -78000,
    expectedShortfall: -92000,
    concentration: 0.67,
    diversificationRatio: 0.78,
    correlationRisk: 0.65,
    liquidityRisk: 0.25,
    marketRisk: 0.85
  })

  const yieldPositions = ref<YieldPosition[]>([
    {
      id: '1',
      protocol: 'Ethereum 2.0',
      asset: 'ETH',
      staked: 64.0,
      value: 210000,
      rewards: 8.2,
      apr: 4.2,
      apy: 4.3,
      autoCompound: false,
      chain: 'ethereum'
    },
    {
      id: '2',
      protocol: 'Uniswap V3',
      asset: 'ETH-USDC',
      pool: '0.3%',
      staked: 150000,
      value: 150000,
      rewards: 750,
      apr: 6.8,
      apy: 7.1,
      autoCompound: true,
      chain: 'ethereum'
    },
    {
      id: '3',
      protocol: 'Aave',
      asset: 'USDC',
      staked: 50000,
      value: 50000,
      rewards: 125,
      apr: 3.2,
      apy: 3.25,
      autoCompound: true,
      chain: 'polygon'
    }
  ])

  // Computed values
  const totalPortfolioValue = computed(() => {
    return positions.value.reduce((total, position) => total + position.value, 0)
  })

  const totalUnrealizedPnL = computed(() => {
    return positions.value.reduce((total, position) => total + position.pnl, 0)
  })

  const totalYieldValue = computed(() => {
    return yieldPositions.value.reduce((total, position) => total + position.value, 0)
  })

  const totalYieldRewards = computed(() => {
    return yieldPositions.value.reduce((total, position) => total + position.rewards, 0)
  })

  const averageAPY = computed(() => {
    if (yieldPositions.value.length === 0) return 0

    const weightedSum = yieldPositions.value.reduce((sum, position) => {
      return sum + (position.apy * position.value)
    }, 0)

    return weightedSum / totalYieldValue.value
  })

  const assetAllocation = computed(() => {
    const allocation: Record<string, number> = {}

    positions.value.forEach(position => {
      const baseAsset = position.symbol.split('-')[0] // Handle LP tokens
      allocation[baseAsset] = (allocation[baseAsset] || 0) + position.value
    })

    return Object.entries(allocation).map(([asset, value]) => ({
      asset,
      value,
      percentage: (value / totalPortfolioValue.value) * 100
    }))
  })

  const chainAllocation = computed(() => {
    const allocation: Record<string, number> = {}

    positions.value.forEach(position => {
      allocation[position.chain] = (allocation[position.chain] || 0) + position.value
    })

    return Object.entries(allocation).map(([chain, value]) => ({
      chain,
      value,
      percentage: (value / totalPortfolioValue.value) * 100
    }))
  })

  const protocolAllocation = computed(() => {
    const allocation: Record<string, number> = {}

    positions.value.forEach(position => {
      const protocol = position.protocol || 'Spot'
      allocation[protocol] = (allocation[protocol] || 0) + position.value
    })

    return Object.entries(allocation).map(([protocol, value]) => ({
      protocol,
      value,
      percentage: (value / totalPortfolioValue.value) * 100
    }))
  })

  const topPerformers = computed(() => {
    return [...positions.value]
      .sort((a, b) => b.pnlPercent - a.pnlPercent)
      .slice(0, 5)
  })

  const topLosers = computed(() => {
    return [...positions.value]
      .sort((a, b) => a.pnlPercent - b.pnlPercent)
      .slice(0, 5)
  })

  const recentTrades = computed(() => {
    return [...trades.value]
      .sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
      .slice(0, 10)
  })

  // Actions
  async function fetchPortfolioData() {
    isLoading.value = true
    try {
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1000))

      // Update last update timestamp
      lastUpdate.value = new Date().toISOString()

      // In real app, this would fetch data from API
      // portfolioMetrics.value = await api.getPortfolioMetrics()
      // positions.value = await api.getPositions()
      // etc.

    } catch (error) {
      console.error('Failed to fetch portfolio data:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  async function addPosition(position: Omit<Position, 'id'>) {
    const newPosition: Position = {
      ...position,
      id: Date.now().toString()
    }
    positions.value.push(newPosition)

    // Recalculate metrics
    await updatePortfolioMetrics()
  }

  async function updatePosition(id: string, updates: Partial<Position>) {
    const index = positions.value.findIndex(p => p.id === id)
    if (index !== -1) {
      positions.value[index] = { ...positions.value[index], ...updates }
      await updatePortfolioMetrics()
    }
  }

  async function removePosition(id: string) {
    const index = positions.value.findIndex(p => p.id === id)
    if (index !== -1) {
      positions.value.splice(index, 1)
      await updatePortfolioMetrics()
    }
  }

  async function addTrade(trade: Omit<Trade, 'id'>) {
    const newTrade: Trade = {
      ...trade,
      id: Date.now().toString()
    }
    trades.value.push(newTrade)

    // Update position if it exists
    if (trade.status === 'completed') {
      await updatePositionFromTrade(newTrade)
    }
  }

  async function updatePositionFromTrade(trade: Trade) {
    const position = positions.value.find(p => p.symbol === trade.symbol)

    if (position) {
      if (trade.type === 'buy') {
        const totalCost = position.quantity * position.avgPrice + trade.quantity * trade.price
        const totalQuantity = position.quantity + trade.quantity

        position.avgPrice = totalCost / totalQuantity
        position.quantity = totalQuantity
      } else {
        position.quantity -= trade.quantity

        if (position.quantity <= 0) {
          await removePosition(position.id)
          return
        }
      }

      position.value = position.quantity * position.currentPrice
      position.pnl = (position.currentPrice - position.avgPrice) * position.quantity
      position.pnlPercent = (position.pnl / (position.avgPrice * position.quantity)) * 100
    } else if (trade.type === 'buy') {
      // Create new position
      await addPosition({
        symbol: trade.symbol,
        name: trade.symbol, // In real app, would fetch name from API
        quantity: trade.quantity,
        avgPrice: trade.price,
        currentPrice: trade.price,
        value: trade.value,
        pnl: 0,
        pnlPercent: 0,
        allocation: 0, // Will be calculated in updatePortfolioMetrics
        dayChange: 0,
        chain: trade.chain,
        type: 'spot'
      })
    }
  }

  async function updatePortfolioMetrics() {
    const totalValue = totalPortfolioValue.value
    const totalPnL = totalUnrealizedPnL.value

    // Update allocation percentages
    positions.value.forEach(position => {
      position.allocation = (position.value / totalValue) * 100
    })

    // Update portfolio metrics
    portfolioMetrics.value.totalValue = totalValue
    portfolioMetrics.value.totalPnL = totalPnL
    portfolioMetrics.value.totalChange = (totalPnL / (totalValue - totalPnL)) * 100
    portfolioMetrics.value.activePositions = positions.value.length

    // Calculate other metrics (simplified)
    const returns = positions.value.map(p => p.pnlPercent)
    const avgReturn = returns.reduce((sum, r) => sum + r, 0) / returns.length
    const returnVariance = returns.reduce((sum, r) => sum + Math.pow(r - avgReturn, 2), 0) / returns.length
    const volatility = Math.sqrt(returnVariance)

    portfolioMetrics.value.volatility = volatility
    portfolioMetrics.value.sharpeRatio = avgReturn / volatility || 0
  }

  async function generatePerformanceHistory(days: number = 30) {
    const history: PerformanceData[] = []
    let baseValue = portfolioMetrics.value.totalValue - portfolioMetrics.value.totalPnL

    for (let i = days - 1; i >= 0; i--) {
      const date = new Date()
      date.setDate(date.getDate() - i)

      // Simulate daily returns
      const dailyReturn = (Math.random() - 0.48) * 0.05 // Slight positive bias
      const dailyValue = baseValue * (1 + dailyReturn)
      const pnl = dailyValue - baseValue

      history.push({
        timestamp: date.toISOString(),
        portfolioValue: dailyValue,
        pnl: pnl,
        returns: dailyReturn * 100,
        benchmark: baseValue * (1 + (Math.random() - 0.5) * 0.03), // Market benchmark
        drawdown: Math.random() * -0.1, // Max 10% drawdown
        volatility: Math.random() * 0.3 + 0.1 // 10-40% volatility
      })

      baseValue = dailyValue
    }

    performanceHistory.value = history
  }

  async function exportData(format: 'json' | 'csv' = 'json') {
    const data = {
      portfolioMetrics: portfolioMetrics.value,
      positions: positions.value,
      trades: trades.value,
      riskMetrics: riskMetrics.value,
      yieldPositions: yieldPositions.value,
      performanceHistory: performanceHistory.value,
      exportedAt: new Date().toISOString()
    }

    if (format === 'json') {
      return JSON.stringify(data, null, 2)
    } else {
      // Convert to CSV format
      // This is a simplified implementation
      const csvData = positions.value.map(position => ({
        Symbol: position.symbol,
        Quantity: position.quantity,
        'Avg Price': position.avgPrice,
        'Current Price': position.currentPrice,
        Value: position.value,
        'P&L': position.pnl,
        'P&L %': position.pnlPercent,
        Allocation: position.allocation
      }))

      const headers = Object.keys(csvData[0])
      const csvString = [
        headers.join(','),
        ...csvData.map(row => headers.map(header => row[header as keyof typeof row]).join(','))
      ].join('\n')

      return csvString
    }
  }

  return {
    // State
    isLoading,
    lastUpdate,
    portfolioMetrics,
    positions,
    trades,
    performanceHistory,
    riskMetrics,
    yieldPositions,

    // Computed
    totalPortfolioValue,
    totalUnrealizedPnL,
    totalYieldValue,
    totalYieldRewards,
    averageAPY,
    assetAllocation,
    chainAllocation,
    protocolAllocation,
    topPerformers,
    topLosers,
    recentTrades,

    // Actions
    fetchPortfolioData,
    addPosition,
    updatePosition,
    removePosition,
    addTrade,
    updatePortfolioMetrics,
    generatePerformanceHistory,
    exportData
  }
})