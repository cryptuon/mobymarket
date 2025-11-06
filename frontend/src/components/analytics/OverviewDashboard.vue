<template>
  <div class="space-y-6">
    <!-- Portfolio Performance Chart -->
    <Grid :cols="{ xs: 1, lg: 2 }" gap="6">
      <GridItem>
        <PortfolioPerformanceChart
          :data="performanceData"
          :time-range="timeRange"
          :loading="loading"
        />
      </GridItem>
      <GridItem>
        <AssetAllocationChart
          :data="allocationData"
          :loading="loading"
        />
      </GridItem>
    </Grid>

    <!-- Key Metrics Grid -->
    <Grid :cols="{ xs: 1, md: 2, lg: 3 }" gap="6">
      <GridItem>
        <PnLBreakdownCard
          :data="pnlBreakdown"
          :loading="loading"
          @drill-down="$emit('metric-drill-down', 'pnl', $event)"
        />
      </GridItem>
      <GridItem>
        <TopPerformersCard
          :data="topPerformers"
          :loading="loading"
        />
      </GridItem>
      <GridItem>
        <RiskMetricsCard
          :data="riskMetrics"
          :loading="loading"
        />
      </GridItem>
    </Grid>

    <!-- Recent Activity & Market Overview -->
    <Grid :cols="{ xs: 1, lg: 3 }" gap="6">
      <GridItem :col-span="{ xs: 1, lg: 2 }">
        <RecentActivityFeed
          :activities="recentActivities"
          :loading="loading"
        />
      </GridItem>
      <GridItem>
        <MarketOverviewCard
          :data="marketOverview"
          :loading="loading"
        />
      </GridItem>
    </Grid>

    <!-- Performance Analytics -->
    <Grid :cols="{ xs: 1, lg: 2 }" gap="6">
      <GridItem>
        <TradingPerformanceChart
          :data="tradingPerformance"
          :time-range="timeRange"
          :loading="loading"
        />
      </GridItem>
      <GridItem>
        <VolatilityAnalysisChart
          :data="volatilityData"
          :time-range="timeRange"
          :loading="loading"
        />
      </GridItem>
    </Grid>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'

import Grid from '@components/ui/Grid.vue'
import GridItem from '@components/ui/GridItem.vue'
import PortfolioPerformanceChart from './charts/PortfolioPerformanceChart.vue'
import AssetAllocationChart from './charts/AssetAllocationChart.vue'
import PnLBreakdownCard from './cards/PnLBreakdownCard.vue'
import TopPerformersCard from './cards/TopPerformersCard.vue'
import RiskMetricsCard from './cards/RiskMetricsCard.vue'
import RecentActivityFeed from './cards/RecentActivityFeed.vue'
import MarketOverviewCard from './cards/MarketOverviewCard.vue'
import TradingPerformanceChart from './charts/TradingPerformanceChart.vue'
import VolatilityAnalysisChart from './charts/VolatilityAnalysisChart.vue'

interface Props {
  data: any
  timeRange: string
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

const emit = defineEmits<{
  'data-export': [data: any]
  'metric-drill-down': [metric: string, value: any]
}>()

// Computed data for charts and cards
const performanceData = computed(() => {
  const days = getTimeRangeDays(props.timeRange)
  return Array.from({ length: days }, (_, i) => {
    const date = new Date()
    date.setDate(date.getDate() - (days - i - 1))

    return {
      timestamp: date.toISOString(),
      portfolioValue: 1000000 + Math.random() * 500000 + i * 1000,
      pnl: (Math.random() - 0.5) * 50000,
      returns: (Math.random() - 0.5) * 5,
      benchmark: 1000000 + i * 800 + Math.random() * 100000
    }
  })
})

const allocationData = computed(() => [
  { asset: 'ETH', value: 450000, percentage: 45, color: '#627EEA', change24h: 3.2 },
  { asset: 'BTC', value: 300000, percentage: 30, color: '#F7931A', change24h: -1.5 },
  { asset: 'USDC', value: 150000, percentage: 15, color: '#2775CA', change24h: 0.1 },
  { asset: 'UNI', value: 60000, percentage: 6, color: '#FF007A', change24h: 8.7 },
  { asset: 'AAVE', value: 40000, percentage: 4, color: '#B6509E', change24h: 5.3 }
])

const pnlBreakdown = computed(() => ({
  realized: 125000,
  unrealized: 87500,
  fees: -12500,
  breakdown: [
    { category: 'Spot Trading', pnl: 85000, percentage: 68 },
    { category: 'DeFi Farming', pnl: 45000, percentage: 36 },
    { category: 'Arbitrage', pnl: 7500, percentage: 6 },
    { category: 'Options', pnl: -12500, percentage: -10 }
  ]
}))

const topPerformers = computed(() => [
  { symbol: 'UNI', name: 'Uniswap', pnl: 25000, returns: 45.7, allocation: 6 },
  { symbol: 'AAVE', name: 'Aave', pnl: 18500, returns: 32.1, allocation: 4 },
  { symbol: 'ETH', name: 'Ethereum', pnl: 67500, returns: 18.9, allocation: 45 },
  { symbol: 'COMP', name: 'Compound', pnl: 8200, returns: 15.3, allocation: 2 },
  { symbol: 'SUSHI', name: 'SushiSwap', pnl: -3200, returns: -8.7, allocation: 3 }
])

const riskMetrics = computed(() => ({
  var95: -45000,
  var99: -78000,
  expectedShortfall: -92000,
  beta: 1.35,
  volatility: 28.5,
  sharpeRatio: 1.85,
  maxDrawdown: 8.5,
  riskScore: 72
}))

const recentActivities = computed(() => [
  {
    id: '1',
    type: 'trade',
    action: 'Buy',
    asset: 'ETH',
    amount: 15.5,
    value: 49600,
    timestamp: new Date(Date.now() - 300000).toISOString(),
    status: 'completed'
  },
  {
    id: '2',
    type: 'yield',
    action: 'Harvest',
    asset: 'UNI-ETH LP',
    amount: 2.3,
    value: 1250,
    timestamp: new Date(Date.now() - 1800000).toISOString(),
    status: 'completed'
  },
  {
    id: '3',
    type: 'trade',
    action: 'Sell',
    asset: 'AAVE',
    amount: 50,
    value: 8500,
    timestamp: new Date(Date.now() - 3600000).toISOString(),
    status: 'completed'
  }
])

const marketOverview = computed(() => ({
  marketCap: 2450000000000,
  volume24h: 125000000000,
  dominance: { btc: 52.3, eth: 18.7 },
  fearGreedIndex: 68,
  volatilityIndex: 0.85,
  trends: [
    { metric: 'DeFi TVL', value: '$85.2B', change: 12.5, trend: 'up' },
    { metric: 'NFT Volume', value: '$2.1B', change: -8.3, trend: 'down' },
    { metric: 'Whale Activity', value: '847', change: 23.7, trend: 'up' }
  ]
}))

const tradingPerformance = computed(() => {
  const days = getTimeRangeDays(props.timeRange)
  return Array.from({ length: days }, (_, i) => {
    const date = new Date()
    date.setDate(date.getDate() - (days - i - 1))

    return {
      timestamp: date.toISOString(),
      winRate: 65 + Math.random() * 20,
      profitFactor: 1.2 + Math.random() * 0.8,
      avgWin: 2500 + Math.random() * 1500,
      avgLoss: -1200 - Math.random() * 800,
      trades: Math.floor(Math.random() * 10) + 1
    }
  })
})

const volatilityData = computed(() => {
  const days = getTimeRangeDays(props.timeRange)
  return Array.from({ length: days }, (_, i) => {
    const date = new Date()
    date.setDate(date.getDate() - (days - i - 1))

    return {
      timestamp: date.toISOString(),
      portfolioVol: 0.15 + Math.random() * 0.3,
      marketVol: 0.25 + Math.random() * 0.4,
      correlation: 0.3 + Math.random() * 0.5,
      beta: 0.8 + Math.random() * 0.8
    }
  })
})

// Helper function
function getTimeRangeDays(range: string): number {
  switch (range) {
    case '24h': return 1
    case '7d': return 7
    case '30d': return 30
    case '90d': return 90
    case '1y': return 365
    default: return 30
  }
}

// Watch for data changes
watch(() => props.timeRange, () => {
  // Trigger data refresh when time range changes
}, { immediate: true })
</script>