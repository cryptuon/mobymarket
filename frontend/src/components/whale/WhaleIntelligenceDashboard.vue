<template>
  <div class="space-y-6">
    <!-- Dashboard Header -->
    <div class="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
      <div>
        <h1 class="text-3xl font-bold text-gradient-primary">Whale Intelligence</h1>
        <p class="text-white/60 mt-1">Real-time tracking and analysis of large market movements</p>
      </div>

      <div class="flex items-center space-x-3">
        <!-- Time Range Filter -->
        <select
          v-model="timeRange"
          class="bg-slate-800/50 border border-slate-600/50 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-moby-500/50"
          @change="refreshData"
        >
          <option value="1h">Last Hour</option>
          <option value="24h">Last 24 Hours</option>
          <option value="7d">Last 7 Days</option>
          <option value="30d">Last 30 Days</option>
        </select>

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

    <!-- Key Metrics Grid -->
    <Grid :cols="{ xs: 1, sm: 2, lg: 4 }" gap="6">
      <GridItem>
        <MetricCard
          title="Active Whales"
          :value="metrics.activeWhales"
          :change="metrics.activeWhalesChange"
          icon="UserGroupIcon"
          color="blue"
        />
      </GridItem>
      <GridItem>
        <MetricCard
          title="Total Volume"
          :value="formatCurrency(metrics.totalVolume)"
          :change="metrics.volumeChange"
          icon="CurrencyDollarIcon"
          color="green"
        />
      </GridItem>
      <GridItem>
        <MetricCard
          title="Largest Trade"
          :value="formatCurrency(metrics.largestTrade)"
          :change="metrics.largestTradeChange"
          icon="TrendingUpIcon"
          color="purple"
        />
      </GridItem>
      <GridItem>
        <MetricCard
          title="Avg Trade Size"
          :value="formatCurrency(metrics.avgTradeSize)"
          :change="metrics.avgTradeSizeChange"
          icon="ScaleIcon"
          color="orange"
        />
      </GridItem>
    </Grid>

    <!-- Main Dashboard Grid -->
    <Grid :cols="{ xs: 1, lg: 3 }" gap="6">
      <!-- Live Activity Feed -->
      <GridItem :col-span="{ xs: 1, lg: 2 }">
        <LiveWhaleActivityFeed @activity-click="selectActivity" />
      </GridItem>

      <!-- Activity Heatmap -->
      <GridItem>
        <WhaleActivityHeatmap :data="heatmapData" />
      </GridItem>
    </Grid>

    <!-- Analysis Grid -->
    <Grid :cols="{ xs: 1, lg: 2 }" gap="6">
      <!-- Top Whales -->
      <GridItem>
        <TopWhalesCard :whales="topWhales" />
      </GridItem>

      <!-- Token Flow Analysis -->
      <GridItem>
        <TokenFlowAnalysis :flows="tokenFlows" />
      </GridItem>
    </Grid>

    <!-- Charts Grid -->
    <Grid :cols="{ xs: 1, lg: 2 }" gap="6">
      <!-- Volume Chart -->
      <GridItem>
        <WhaleVolumeChart
          :data="volumeChartData"
          :timeRange="timeRange"
        />
      </GridItem>

      <!-- Network Distribution -->
      <GridItem>
        <NetworkDistributionChart :data="networkData" />
      </GridItem>
    </Grid>

    <!-- Activity Details Modal -->
    <WhaleActivityModal
      v-if="selectedActivity"
      :activity="selectedActivity"
      @close="selectedActivity = null"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'

import Grid from '@components/ui/Grid.vue'
import GridItem from '@components/ui/GridItem.vue'
import Button from '@components/ui/Button.vue'
import MetricCard from '@components/dashboard/MetricCard.vue'
import LiveWhaleActivityFeed from './LiveWhaleActivityFeed.vue'
import WhaleActivityHeatmap from './WhaleActivityHeatmap.vue'
import TopWhalesCard from './TopWhalesCard.vue'
import TokenFlowAnalysis from './TokenFlowAnalysis.vue'
import WhaleVolumeChart from './WhaleVolumeChart.vue'
import NetworkDistributionChart from './NetworkDistributionChart.vue'
import WhaleActivityModal from './WhaleActivityModal.vue'

import { useRealTimeData } from '@/composables/useRealTimeData'
import { useBreakpoints } from '@/composables/useBreakpoints'
import type { WhaleActivity } from '@/types'

const { liveWhaleActivity, isConnected } = useRealTimeData()
const { isMobile } = useBreakpoints()

const timeRange = ref('24h')
const isLoading = ref(false)
const selectedActivity = ref<WhaleActivity | null>(null)

// Dashboard metrics
const metrics = ref({
  activeWhales: 247,
  activeWhalesChange: 12.5,
  totalVolume: 2850000000,
  volumeChange: 8.3,
  largestTrade: 15000000,
  largestTradeChange: -5.2,
  avgTradeSize: 1250000,
  avgTradeSizeChange: 15.7
})

// Computed data for charts
const heatmapData = computed(() => {
  // Generate heatmap data based on activity
  const hours = Array.from({ length: 24 }, (_, i) => i)
  const days = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']

  return days.map(day => ({
    day,
    hours: hours.map(hour => ({
      hour,
      value: Math.floor(Math.random() * 100),
      count: Math.floor(Math.random() * 50)
    }))
  }))
})

const topWhales = computed(() => [
  {
    id: '1',
    address: '0x1234...5678',
    volume24h: 125000000,
    trades: 15,
    winRate: 78.5,
    avgSize: 8333333,
    lastActive: '2 min ago',
    rank: 1
  },
  {
    id: '2',
    address: '0x9abc...def0',
    volume24h: 98000000,
    trades: 8,
    winRate: 85.2,
    avgSize: 12250000,
    lastActive: '15 min ago',
    rank: 2
  },
  {
    id: '3',
    address: '0x3456...789a',
    volume24h: 87000000,
    trades: 22,
    winRate: 65.8,
    avgSize: 3954545,
    lastActive: '1 hr ago',
    rank: 3
  }
])

const tokenFlows = computed(() => [
  {
    token: 'ETH',
    netFlow: 2500000,
    inflowVolume: 15000000,
    outflowVolume: 12500000,
    change24h: 12.5
  },
  {
    token: 'BTC',
    netFlow: -1800000,
    inflowVolume: 8200000,
    outflowVolume: 10000000,
    change24h: -8.3
  },
  {
    token: 'USDC',
    netFlow: 5000000,
    inflowVolume: 25000000,
    outflowVolume: 20000000,
    change24h: 25.2
  }
])

const volumeChartData = computed(() => {
  // Generate time series data based on selected time range
  const dataPoints = timeRange.value === '1h' ? 60 :
                     timeRange.value === '24h' ? 24 :
                     timeRange.value === '7d' ? 7 : 30

  return Array.from({ length: dataPoints }, (_, i) => ({
    timestamp: new Date(Date.now() - (dataPoints - i) * getTimeInterval()).toISOString(),
    volume: Math.random() * 100000000 + 50000000,
    trades: Math.floor(Math.random() * 100) + 20,
    whales: Math.floor(Math.random() * 50) + 10
  }))
})

const networkData = computed(() => [
  { network: 'Ethereum', volume: 1500000000, percentage: 52.6, color: '#627EEA' },
  { network: 'Arbitrum', volume: 650000000, percentage: 22.8, color: '#28A0F0' },
  { network: 'Polygon', volume: 420000000, percentage: 14.7, color: '#8247E5' },
  { network: 'Optimism', volume: 180000000, percentage: 6.3, color: '#FF0420' },
  { network: 'Base', volume: 100000000, percentage: 3.5, color: '#0052FF' }
])

// Methods
function getTimeInterval(): number {
  switch (timeRange.value) {
    case '1h': return 60 * 1000 // 1 minute intervals
    case '24h': return 60 * 60 * 1000 // 1 hour intervals
    case '7d': return 24 * 60 * 60 * 1000 // 1 day intervals
    case '30d': return 24 * 60 * 60 * 1000 // 1 day intervals
    default: return 60 * 60 * 1000
  }
}

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
    await new Promise(resolve => setTimeout(resolve, 1000))

    // Update metrics with some randomization
    metrics.value = {
      activeWhales: Math.floor(Math.random() * 100) + 200,
      activeWhalesChange: (Math.random() - 0.5) * 30,
      totalVolume: Math.random() * 2000000000 + 1000000000,
      volumeChange: (Math.random() - 0.5) * 50,
      largestTrade: Math.random() * 20000000 + 5000000,
      largestTradeChange: (Math.random() - 0.5) * 40,
      avgTradeSize: Math.random() * 2000000 + 500000,
      avgTradeSizeChange: (Math.random() - 0.5) * 50
    }
  } catch (error) {
    console.error('Failed to refresh data:', error)
  } finally {
    isLoading.value = false
  }
}

function selectActivity(activity: WhaleActivity) {
  selectedActivity.value = activity
}

// Auto-refresh data periodically
onMounted(() => {
  const refreshInterval = setInterval(refreshData, 30000) // Every 30 seconds

  return () => clearInterval(refreshInterval)
})

// Refresh when time range changes
watch(timeRange, refreshData)
</script>

<style scoped>
.text-gradient-primary {
  background: linear-gradient(135deg, #0ea5e9 0%, #8b5cf6 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}
</style>