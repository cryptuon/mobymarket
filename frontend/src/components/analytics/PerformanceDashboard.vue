<template>
  <div class="space-y-6">
    <!-- Performance Summary -->
    <Grid :cols="{ xs: 2, lg: 5 }" gap="6">
      <GridItem>
        <MetricCard
          title="Total Return"
          :value="`${performanceMetrics.totalReturn >= 0 ? '+' : ''}${performanceMetrics.totalReturn.toFixed(1)}%`"
          :change="performanceMetrics.totalReturnChange"
          icon="TrendingUpIcon"
          :color="performanceMetrics.totalReturn >= 0 ? 'green' : 'red'"
        />
      </GridItem>
      <GridItem>
        <MetricCard
          title="Annualized Return"
          :value="`${performanceMetrics.annualizedReturn >= 0 ? '+' : ''}${performanceMetrics.annualizedReturn.toFixed(1)}%`"
          :change="performanceMetrics.annualizedChange"
          icon="CalendarIcon"
          :color="performanceMetrics.annualizedReturn >= 15 ? 'green' : performanceMetrics.annualizedReturn >= 8 ? 'yellow' : 'red'"
        />
      </GridItem>
      <GridItem>
        <MetricCard
          title="Sharpe Ratio"
          :value="performanceMetrics.sharpeRatio.toFixed(2)"
          :change="performanceMetrics.sharpeChange"
          icon="CalculatorIcon"
          :color="performanceMetrics.sharpeRatio >= 1.5 ? 'green' : performanceMetrics.sharpeRatio >= 1 ? 'yellow' : 'red'"
        />
      </GridItem>
      <GridItem>
        <MetricCard
          title="Alpha"
          :value="`${performanceMetrics.alpha >= 0 ? '+' : ''}${performanceMetrics.alpha.toFixed(2)}`"
          :change="performanceMetrics.alphaChange"
          icon="SparklesIcon"
          :color="performanceMetrics.alpha >= 0 ? 'green' : 'red'"
        />
      </GridItem>
      <GridItem>
        <MetricCard
          title="Win Rate"
          :value="`${performanceMetrics.winRate.toFixed(1)}%`"
          :change="performanceMetrics.winRateChange"
          icon="TrophyIcon"
          :color="performanceMetrics.winRate >= 70 ? 'green' : performanceMetrics.winRate >= 50 ? 'yellow' : 'red'"
        />
      </GridItem>
    </Grid>

    <!-- Performance Charts -->
    <Grid :cols="{ xs: 1, lg: 2 }" gap="6">
      <GridItem>
        <PerformanceChart
          :data="performanceChartData"
          :time-range="timeRange"
          :show-benchmark="true"
          :loading="loading"
        />
      </GridItem>
      <GridItem>
        <ReturnsDistributionChart
          :data="returnsDistribution"
          :loading="loading"
        />
      </GridItem>
    </Grid>

    <!-- Detailed Metrics -->
    <Grid :cols="{ xs: 1, lg: 3 }" gap="6">
      <GridItem>
        <RiskAdjustedReturnsCard
          :data="riskAdjustedMetrics"
          :loading="loading"
        />
      </GridItem>
      <GridItem>
        <DrawdownAnalysisCard
          :data="drawdownData"
          :loading="loading"
        />
      </GridItem>
      <GridItem>
        <BenchmarkComparisonCard
          :data="benchmarkComparison"
          :loading="loading"
        />
      </GridItem>
    </Grid>

    <!-- Trade Analysis -->
    <Grid :cols="{ xs: 1, lg: 2 }" gap="6">
      <GridItem>
        <TradeAnalysisCard
          :data="tradeAnalysis"
          :loading="loading"
        />
      </GridItem>
      <GridItem>
        <MonthlyReturnsHeatmap
          :data="monthlyReturns"
          :loading="loading"
        />
      </GridItem>
    </Grid>

    <!-- Asset Performance -->
    <Grid :cols="{ xs: 1, lg: 2 }" gap="6">
      <GridItem>
        <AssetPerformanceTable
          :assets="assetPerformance"
          :loading="loading"
          @asset-drill-down="handleAssetDrillDown"
        />
      </GridItem>
      <GridItem>
        <PerformanceAttributionChart
          :data="attributionData"
          :loading="loading"
        />
      </GridItem>
    </Grid>

    <!-- Time Period Analysis -->
    <TimePeriodAnalysis
      :data="timePeriodData"
      :loading="loading"
      @period-selected="handlePeriodSelected"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

import Grid from '@components/ui/Grid.vue'
import GridItem from '@components/ui/GridItem.vue'
import MetricCard from '@components/dashboard/MetricCard.vue'
import PerformanceChart from './charts/PerformanceChart.vue'
import ReturnsDistributionChart from './charts/ReturnsDistributionChart.vue'
import RiskAdjustedReturnsCard from './cards/RiskAdjustedReturnsCard.vue'
import DrawdownAnalysisCard from './cards/DrawdownAnalysisCard.vue'
import BenchmarkComparisonCard from './cards/BenchmarkComparisonCard.vue'
import TradeAnalysisCard from './cards/TradeAnalysisCard.vue'
import MonthlyReturnsHeatmap from './charts/MonthlyReturnsHeatmap.vue'
import AssetPerformanceTable from './cards/AssetPerformanceTable.vue'
import PerformanceAttributionChart from './charts/PerformanceAttributionChart.vue'
import TimePeriodAnalysis from './cards/TimePeriodAnalysis.vue'

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

// Performance metrics
const performanceMetrics = ref({
  totalReturn: 24.7,
  totalReturnChange: 2.3,
  annualizedReturn: 18.5,
  annualizedChange: 1.2,
  sharpeRatio: 1.85,
  sharpeChange: 0.15,
  alpha: 0.08,
  alphaChange: 0.02,
  winRate: 73.8,
  winRateChange: 2.1
})

// Performance chart data
const performanceChartData = computed(() => {
  const days = getTimeRangeDays(props.timeRange)
  let portfolioValue = 1000000
  let benchmarkValue = 1000000

  return Array.from({ length: days }, (_, i) => {
    const date = new Date()
    date.setDate(date.getDate() - (days - i - 1))

    // Portfolio slightly outperforms benchmark
    const portfolioReturn = (Math.random() - 0.47) * 0.02 // Slight positive bias
    const benchmarkReturn = (Math.random() - 0.48) * 0.02

    portfolioValue *= (1 + portfolioReturn)
    benchmarkValue *= (1 + benchmarkReturn)

    return {
      timestamp: date.toISOString(),
      portfolio: portfolioValue,
      benchmark: benchmarkValue,
      portfolioReturn: portfolioReturn * 100,
      benchmarkReturn: benchmarkReturn * 100,
      outperformance: (portfolioReturn - benchmarkReturn) * 100
    }
  })
})

// Returns distribution
const returnsDistribution = computed(() => {
  const returns = performanceChartData.value.map(d => d.portfolioReturn)
  const bins = 20
  const min = Math.min(...returns)
  const max = Math.max(...returns)
  const binWidth = (max - min) / bins

  const histogram = Array.from({ length: bins }, (_, i) => {
    const binStart = min + i * binWidth
    const binEnd = binStart + binWidth
    const count = returns.filter(r => r >= binStart && r < binEnd).length

    return {
      range: `${binStart.toFixed(1)}% to ${binEnd.toFixed(1)}%`,
      count,
      frequency: count / returns.length
    }
  })

  return histogram
})

// Risk-adjusted metrics
const riskAdjustedMetrics = ref({
  sharpe: 1.85,
  sortino: 2.31,
  calmar: 1.48,
  treynor: 0.125,
  informationRatio: 0.68,
  trackingError: 0.045,
  maxDrawdown: 8.5,
  volatility: 18.2
})

// Drawdown data
const drawdownData = computed(() => {
  const chartData = performanceChartData.value
  let peak = chartData[0]?.portfolio || 1000000

  return chartData.map(point => {
    if (point.portfolio > peak) {
      peak = point.portfolio
    }

    const drawdown = ((point.portfolio - peak) / peak) * 100

    return {
      timestamp: point.timestamp,
      drawdown,
      portfolio: point.portfolio,
      peak
    }
  })
})

// Benchmark comparison
const benchmarkComparison = ref({
  portfolio: {
    return: 24.7,
    volatility: 18.2,
    sharpe: 1.85,
    maxDrawdown: 8.5
  },
  benchmark: {
    return: 18.3,
    volatility: 15.8,
    sharpe: 1.16,
    maxDrawdown: 12.1
  },
  outperformance: 6.4,
  correlation: 0.78,
  beta: 1.35
})

// Trade analysis
const tradeAnalysis = ref({
  totalTrades: 247,
  winningTrades: 182,
  losingTrades: 65,
  winRate: 73.7,
  avgWin: 3250,
  avgLoss: -1180,
  profitFactor: 2.75,
  largestWin: 15600,
  largestLoss: -8200,
  avgHoldTime: '3.2 days',
  tradingFrequency: 'Active'
})

// Monthly returns heatmap
const monthlyReturns = computed(() => {
  const months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec']
  const years = [2023, 2024]

  return years.map(year => ({
    year,
    months: months.map(month => ({
      month,
      return: (Math.random() - 0.4) * 20, // Slight positive bias
      trades: Math.floor(Math.random() * 30) + 5
    }))
  }))
})

// Asset performance
const assetPerformance = ref([
  {
    symbol: 'ETH',
    name: 'Ethereum',
    allocation: 46.1,
    return: 28.5,
    contribution: 13.1,
    volatility: 22.3,
    sharpe: 1.28,
    beta: 1.45
  },
  {
    symbol: 'BTC',
    name: 'Bitcoin',
    allocation: 28.8,
    return: 22.1,
    contribution: 6.4,
    volatility: 18.7,
    sharpe: 1.18,
    beta: 1.15
  },
  {
    symbol: 'UNI',
    name: 'Uniswap',
    allocation: 8.1,
    return: 45.7,
    contribution: 3.7,
    volatility: 45.2,
    sharpe: 1.01,
    beta: 2.15
  },
  {
    symbol: 'AAVE',
    name: 'Aave',
    allocation: 5.0,
    return: 32.1,
    contribution: 1.6,
    volatility: 38.9,
    sharpe: 0.83,
    beta: 1.85
  }
])

// Performance attribution
const attributionData = ref([
  { category: 'Asset Selection', contribution: 8.5 },
  { category: 'Sector Allocation', contribution: 3.2 },
  { category: 'Market Timing', contribution: 2.1 },
  { category: 'Currency Effect', contribution: 0.8 },
  { category: 'Interaction Effect', contribution: -0.9 }
])

// Time period analysis
const timePeriodData = ref([
  { period: '1 Day', return: 0.8, volatility: 2.1, sharpe: 0.38 },
  { period: '1 Week', return: 2.3, volatility: 5.8, sharpe: 0.40 },
  { period: '1 Month', return: 4.7, volatility: 12.5, sharpe: 0.38 },
  { period: '3 Months', return: 12.1, volatility: 18.2, sharpe: 0.66 },
  { period: '6 Months', return: 18.5, volatility: 19.8, sharpe: 0.93 },
  { period: '1 Year', return: 24.7, volatility: 18.2, sharpe: 1.36 },
  { period: 'YTD', return: 22.1, volatility: 17.9, sharpe: 1.23 },
  { period: 'All Time', return: 156.8, volatility: 22.1, sharpe: 1.42 }
])

// Methods
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

function handleAssetDrillDown(asset: any) {
  emit('metric-drill-down', 'asset-performance', asset)
}

function handlePeriodSelected(period: string) {
  emit('metric-drill-down', 'time-period', period)
}
</script>