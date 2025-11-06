<template>
  <div class="space-y-6">
    <!-- Dashboard Header -->
    <div class="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
      <div>
        <h1 class="text-3xl font-bold text-gradient-primary">Analytics Dashboard</h1>
        <p class="text-white/60 mt-1">Comprehensive market insights and performance tracking</p>
      </div>

      <div class="flex items-center space-x-3">
        <!-- Dashboard Type Selector -->
        <select
          v-model="dashboardType"
          class="bg-slate-800/50 border border-slate-600/50 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-moby-500/50"
        >
          <option value="overview">Overview</option>
          <option value="portfolio">Portfolio</option>
          <option value="market">Market Analysis</option>
          <option value="risk">Risk Assessment</option>
          <option value="performance">Performance</option>
        </select>

        <!-- Time Range -->
        <select
          v-model="timeRange"
          class="bg-slate-800/50 border border-slate-600/50 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-moby-500/50"
        >
          <option value="24h">24 Hours</option>
          <option value="7d">7 Days</option>
          <option value="30d">30 Days</option>
          <option value="90d">90 Days</option>
          <option value="1y">1 Year</option>
        </select>

        <!-- Export Button -->
        <Button
          @click="exportData"
          variant="outline"
          size="md"
          icon-left="DocumentArrowDownIcon"
        >
          Export
        </Button>

        <!-- Refresh Button -->
        <Button
          @click="refreshData"
          :loading="isLoading"
          variant="outline"
          size="md"
          icon-left="ArrowPathIcon"
        >
          Refresh
        </Button>
      </div>
    </div>

    <!-- Quick Stats Bar -->
    <div class="grid grid-cols-2 lg:grid-cols-6 gap-4">
      <MetricCard
        title="Total PnL"
        :value="formatCurrency(analytics.totalPnL)"
        :change="analytics.pnlChange"
        icon="CurrencyDollarIcon"
        :color="analytics.totalPnL >= 0 ? 'green' : 'red'"
        :highlight="Math.abs(analytics.pnlChange) > 10"
      />
      <MetricCard
        title="Win Rate"
        :value="`${analytics.winRate}%`"
        :change="analytics.winRateChange"
        icon="TrophyIcon"
        :color="analytics.winRate >= 70 ? 'green' : analytics.winRate >= 50 ? 'yellow' : 'red'"
      />
      <MetricCard
        title="Total Volume"
        :value="formatCurrency(analytics.totalVolume)"
        :change="analytics.volumeChange"
        icon="ChartBarIcon"
        color="blue"
      />
      <MetricCard
        title="Active Positions"
        :value="analytics.activePositions"
        :change="analytics.positionsChange"
        icon="CubeIcon"
        color="purple"
      />
      <MetricCard
        title="Sharpe Ratio"
        :value="analytics.sharpeRatio.toFixed(2)"
        :change="analytics.sharpeChange"
        icon="CalculatorIcon"
        :color="analytics.sharpeRatio >= 1.5 ? 'green' : analytics.sharpeRatio >= 1 ? 'yellow' : 'red'"
      />
      <MetricCard
        title="Max Drawdown"
        :value="`${analytics.maxDrawdown}%`"
        :change="analytics.drawdownChange"
        icon="ExclamationTriangleIcon"
        :color="analytics.maxDrawdown <= 10 ? 'green' : analytics.maxDrawdown <= 20 ? 'yellow' : 'red'"
      />
    </div>

    <!-- Main Content Based on Dashboard Type -->
    <component
      :is="currentDashboardComponent"
      :data="currentDashboardData"
      :time-range="timeRange"
      :loading="isLoading"
      @data-export="handleDataExport"
      @metric-drill-down="handleMetricDrillDown"
    />

    <!-- AI Insights Panel -->
    <AIInsightsPanel
      :insights="aiInsights"
      :loading="isGeneratingInsights"
      @refresh-insights="generateInsights"
      @apply-suggestion="applySuggestion"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'

import Button from '@components/ui/Button.vue'
import MetricCard from '@components/dashboard/MetricCard.vue'
import OverviewDashboard from './OverviewDashboard.vue'
import PortfolioDashboard from './PortfolioDashboard.vue'
import MarketAnalysisDashboard from './MarketAnalysisDashboard.vue'
import RiskAssessmentDashboard from './RiskAssessmentDashboard.vue'
import PerformanceDashboard from './PerformanceDashboard.vue'
import AIInsightsPanel from './AIInsightsPanel.vue'

import { useAnalyticsStore } from '@/stores/analytics'
import { useNotificationStore } from '@/stores/notifications'

const analyticsStore = useAnalyticsStore()
const notificationStore = useNotificationStore()

const dashboardType = ref<'overview' | 'portfolio' | 'market' | 'risk' | 'performance'>('overview')
const timeRange = ref('30d')
const isLoading = ref(false)
const isGeneratingInsights = ref(false)

// Mock analytics data
const analytics = ref({
  totalPnL: 2456789.50,
  pnlChange: 12.5,
  winRate: 73.8,
  winRateChange: 2.1,
  totalVolume: 45678901.25,
  volumeChange: 8.7,
  activePositions: 23,
  positionsChange: -2.3,
  sharpeRatio: 1.85,
  sharpeChange: 0.15,
  maxDrawdown: 8.5,
  drawdownChange: -1.2
})

// AI Insights mock data
const aiInsights = ref([
  {
    id: '1',
    type: 'opportunity',
    title: 'High-Probability Long Setup',
    description: 'ETH showing strong accumulation pattern with whale inflows increasing 45% in the last 24h.',
    confidence: 87,
    action: 'Consider increasing ETH allocation',
    expectedReturn: 15.3,
    riskLevel: 'medium',
    timeframe: '7-14 days'
  },
  {
    id: '2',
    type: 'warning',
    title: 'Overexposure Risk Detected',
    description: 'Your DeFi allocation is 67% above recommended levels for current market volatility.',
    confidence: 92,
    action: 'Reduce DeFi exposure by 20-25%',
    expectedReturn: -8.2,
    riskLevel: 'high',
    timeframe: 'immediate'
  },
  {
    id: '3',
    type: 'insight',
    title: 'Correlation Alert',
    description: 'Your top 3 positions show 85% correlation - consider diversification.',
    confidence: 78,
    action: 'Add uncorrelated assets',
    expectedReturn: 5.7,
    riskLevel: 'low',
    timeframe: '2-4 weeks'
  }
])

// Computed properties
const currentDashboardComponent = computed(() => {
  switch (dashboardType.value) {
    case 'portfolio': return PortfolioDashboard
    case 'market': return MarketAnalysisDashboard
    case 'risk': return RiskAssessmentDashboard
    case 'performance': return PerformanceDashboard
    default: return OverviewDashboard
  }
})

const currentDashboardData = computed(() => {
  // Return relevant data based on dashboard type
  return {
    timeRange: timeRange.value,
    analytics: analytics.value,
    // Additional data would be loaded based on dashboard type
  }
})

// Methods
function formatCurrency(amount: number): string {
  if (amount >= 1e9) return `$${(amount / 1e9).toFixed(2)}B`
  if (amount >= 1e6) return `$${(amount / 1e6).toFixed(2)}M`
  if (amount >= 1e3) return `$${(amount / 1e3).toFixed(2)}K`
  return `$${amount.toFixed(2)}`
}

async function refreshData() {
  if (isLoading.value) return

  isLoading.value = true
  try {
    // Simulate API call
    await new Promise(resolve => setTimeout(resolve, 1500))

    // Update analytics with some randomization
    analytics.value = {
      totalPnL: analytics.value.totalPnL + (Math.random() - 0.5) * 100000,
      pnlChange: (Math.random() - 0.5) * 20,
      winRate: Math.max(0, Math.min(100, analytics.value.winRate + (Math.random() - 0.5) * 10)),
      winRateChange: (Math.random() - 0.5) * 5,
      totalVolume: analytics.value.totalVolume + (Math.random() - 0.5) * 5000000,
      volumeChange: (Math.random() - 0.5) * 15,
      activePositions: Math.max(0, analytics.value.activePositions + Math.floor((Math.random() - 0.5) * 6)),
      positionsChange: (Math.random() - 0.5) * 10,
      sharpeRatio: Math.max(0, analytics.value.sharpeRatio + (Math.random() - 0.5) * 0.5),
      sharpeChange: (Math.random() - 0.5) * 0.3,
      maxDrawdown: Math.max(0, analytics.value.maxDrawdown + (Math.random() - 0.5) * 5),
      drawdownChange: (Math.random() - 0.5) * 3
    }

    notificationStore.notifySystem(
      'Data Refreshed',
      'Analytics data has been updated',
      'success'
    )
  } catch (error) {
    notificationStore.notifySystem(
      'Refresh Failed',
      'Failed to refresh analytics data',
      'error'
    )
  } finally {
    isLoading.value = false
  }
}

async function exportData() {
  try {
    notificationStore.notifySystem(
      'Exporting Data',
      'Preparing analytics report...',
      'info'
    )

    // Simulate export process
    await new Promise(resolve => setTimeout(resolve, 2000))

    // Create and trigger download
    const data = {
      dashboard: dashboardType.value,
      timeRange: timeRange.value,
      analytics: analytics.value,
      exportedAt: new Date().toISOString()
    }

    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `moby-analytics-${dashboardType.value}-${timeRange.value}-${Date.now()}.json`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)

    notificationStore.notifySystem(
      'Export Complete',
      'Analytics report downloaded successfully',
      'success'
    )
  } catch (error) {
    notificationStore.notifySystem(
      'Export Failed',
      'Failed to export analytics data',
      'error'
    )
  }
}

async function generateInsights() {
  if (isGeneratingInsights.value) return

  isGeneratingInsights.value = true
  try {
    // Simulate AI analysis
    await new Promise(resolve => setTimeout(resolve, 3000))

    // Generate new insights (in real app, this would call AI service)
    const newInsights = [
      {
        id: Date.now().toString(),
        type: 'opportunity',
        title: 'Arbitrage Opportunity Detected',
        description: 'Price discrepancy between Ethereum and Arbitrum for USDC pools.',
        confidence: 89,
        action: 'Execute cross-chain arbitrage',
        expectedReturn: 3.2,
        riskLevel: 'low',
        timeframe: '1-2 hours'
      }
    ]

    aiInsights.value = [...newInsights, ...aiInsights.value.slice(0, 2)]

    notificationStore.notifySystem(
      'Insights Updated',
      'New AI insights generated',
      'success'
    )
  } catch (error) {
    notificationStore.notifySystem(
      'Insights Failed',
      'Failed to generate new insights',
      'error'
    )
  } finally {
    isGeneratingInsights.value = false
  }
}

function applySuggestion(insight: any) {
  notificationStore.notifySystem(
    'Applying Suggestion',
    `Implementing: ${insight.title}`,
    'info'
  )
}

function handleDataExport(data: any) {
  // Handle specific data export requests from child components
  console.log('Exporting specific data:', data)
}

function handleMetricDrillDown(metric: string, value: any) {
  // Handle drill-down requests from child components
  console.log('Drilling down into metric:', metric, value)
}

// Auto-refresh data periodically
onMounted(() => {
  const refreshInterval = setInterval(() => {
    if (!isLoading.value) {
      refreshData()
    }
  }, 60000) // Every minute

  return () => clearInterval(refreshInterval)
})

// Refresh when time range or dashboard type changes
watch([timeRange, dashboardType], () => {
  refreshData()
})
</script>

<style scoped>
.text-gradient-primary {
  background: linear-gradient(135deg, #0ea5e9 0%, #8b5cf6 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}
</style>