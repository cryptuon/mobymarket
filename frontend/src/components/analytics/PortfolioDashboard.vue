<template>
  <div class="space-y-6">
    <!-- Portfolio Summary -->
    <Grid :cols="{ xs: 1, lg: 4 }" gap="6">
      <GridItem>
        <Card variant="glass">
          <div class="p-6">
            <div class="flex items-center space-x-3 mb-4">
              <HeroIcon name="WalletIcon" class="w-6 h-6 text-green-400" />
              <h3 class="text-lg font-semibold text-white">Total Value</h3>
            </div>
            <div class="text-3xl font-bold text-white mb-2">${{ formatCurrency(portfolioSummary.totalValue) }}</div>
            <div class="flex items-center space-x-2">
              <span :class="getChangeColor(portfolioSummary.totalChange)" class="text-sm font-medium">
                {{ portfolioSummary.totalChange >= 0 ? '+' : '' }}{{ portfolioSummary.totalChange.toFixed(2) }}%
              </span>
              <span class="text-xs text-white/60">24h</span>
            </div>
          </div>
        </Card>
      </GridItem>

      <GridItem>
        <Card variant="glass">
          <div class="p-6">
            <div class="flex items-center space-x-3 mb-4">
              <HeroIcon name="TrendingUpIcon" class="w-6 h-6 text-blue-400" />
              <h3 class="text-lg font-semibold text-white">Available Cash</h3>
            </div>
            <div class="text-3xl font-bold text-white mb-2">${{ formatCurrency(portfolioSummary.availableCash) }}</div>
            <div class="text-xs text-white/60">
              {{ ((portfolioSummary.availableCash / portfolioSummary.totalValue) * 100).toFixed(1) }}% of portfolio
            </div>
          </div>
        </Card>
      </GridItem>

      <GridItem>
        <Card variant="glass">
          <div class="p-6">
            <div class="flex items-center space-x-3 mb-4">
              <HeroIcon name="ChartBarIcon" class="w-6 h-6 text-purple-400" />
              <h3 class="text-lg font-semibold text-white">Day's P&L</h3>
            </div>
            <div class="text-3xl font-bold text-white mb-2">
              <span :class="getChangeColor(portfolioSummary.dayPnL)">
                {{ portfolioSummary.dayPnL >= 0 ? '+' : '' }}${{ Math.abs(portfolioSummary.dayPnL).toLocaleString() }}
              </span>
            </div>
            <div class="text-xs text-white/60">Since market open</div>
          </div>
        </Card>
      </GridItem>

      <GridItem>
        <Card variant="glass">
          <div class="p-6">
            <div class="flex items-center space-x-3 mb-4">
              <HeroIcon name="CubeIcon" class="w-6 h-6 text-orange-400" />
              <h3 class="text-lg font-semibold text-white">Positions</h3>
            </div>
            <div class="text-3xl font-bold text-white mb-2">{{ portfolioSummary.totalPositions }}</div>
            <div class="text-xs text-white/60">
              {{ portfolioSummary.profitablePositions }} profitable
            </div>
          </div>
        </Card>
      </GridItem>
    </Grid>

    <!-- Holdings Table & Allocation Chart -->
    <Grid :cols="{ xs: 1, lg: 3 }" gap="6">
      <GridItem :col-span="{ xs: 1, lg: 2 }">
        <PortfolioHoldingsTable
          :holdings="holdings"
          :loading="loading"
          @position-action="handlePositionAction"
          @sort-change="handleSortChange"
        />
      </GridItem>
      <GridItem>
        <PortfolioAllocationChart
          :data="allocationData"
          :loading="loading"
          @rebalance="handleRebalance"
        />
      </GridItem>
    </Grid>

    <!-- Performance & Risk Analysis -->
    <Grid :cols="{ xs: 1, lg: 2 }" gap="6">
      <GridItem>
        <PortfolioPerformanceChart
          :data="performanceHistory"
          :time-range="timeRange"
          :benchmark="true"
          :loading="loading"
        />
      </GridItem>
      <GridItem>
        <RiskAnalysisCard
          :data="riskAnalysis"
          :loading="loading"
          @stress-test="handleStressTest"
        />
      </GridItem>
    </Grid>

    <!-- Diversification & Correlation -->
    <Grid :cols="{ xs: 1, lg: 2 }" gap="6">
      <GridItem>
        <DiversificationAnalysis
          :data="diversificationData"
          :loading="loading"
        />
      </GridItem>
      <GridItem>
        <CorrelationMatrix
          :data="correlationData"
          :loading="loading"
        />
      </GridItem>
    </Grid>

    <!-- Yield & Staking Overview -->
    <YieldStakingOverview
      :data="yieldData"
      :loading="loading"
      @claim-rewards="handleClaimRewards"
      @stake-action="handleStakeAction"
    />

    <!-- Transaction History -->
    <TransactionHistory
      :transactions="transactionHistory"
      :loading="loading"
      @export-transactions="handleExportTransactions"
      @filter-change="handleFilterChange"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

import Grid from '@components/ui/Grid.vue'
import GridItem from '@components/ui/GridItem.vue'
import Card from '@components/ui/Card.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'
import PortfolioHoldingsTable from './portfolio/PortfolioHoldingsTable.vue'
import PortfolioAllocationChart from './portfolio/PortfolioAllocationChart.vue'
import PortfolioPerformanceChart from './charts/PortfolioPerformanceChart.vue'
import RiskAnalysisCard from './portfolio/RiskAnalysisCard.vue'
import DiversificationAnalysis from './portfolio/DiversificationAnalysis.vue'
import CorrelationMatrix from './portfolio/CorrelationMatrix.vue'
import YieldStakingOverview from './portfolio/YieldStakingOverview.vue'
import TransactionHistory from './portfolio/TransactionHistory.vue'

import { useNotificationStore } from '@/stores/notifications'

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

const notificationStore = useNotificationStore()

// Portfolio summary data
const portfolioSummary = ref({
  totalValue: 1000000,
  availableCash: 150000,
  dayPnL: 12500,
  totalChange: 2.3,
  totalPositions: 8,
  profitablePositions: 6
})

// Holdings data
const holdings = ref([
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
    icon: '/tokens/eth.svg'
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
    icon: '/tokens/btc.svg'
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
    icon: '/tokens/usdc.svg'
  }
])

// Allocation data
const allocationData = computed(() =>
  holdings.value.map(holding => ({
    asset: holding.symbol,
    value: holding.value,
    percentage: holding.allocation,
    color: getAssetColor(holding.symbol),
    change24h: holding.dayChange
  }))
)

// Performance history
const performanceHistory = computed(() => {
  const days = getTimeRangeDays(props.timeRange)
  let baseValue = 950000

  return Array.from({ length: days }, (_, i) => {
    const date = new Date()
    date.setDate(date.getDate() - (days - i - 1))

    baseValue += (Math.random() - 0.48) * 5000 // Slight upward trend

    return {
      timestamp: date.toISOString(),
      portfolioValue: baseValue,
      benchmark: 950000 + i * 150 + Math.random() * 2000,
      returns: ((baseValue - 950000) / 950000) * 100
    }
  })
})

// Risk analysis data
const riskAnalysis = ref({
  var95: -45000,
  var99: -78000,
  expectedShortfall: -92000,
  beta: 1.35,
  volatility: 28.5,
  sharpeRatio: 1.85,
  maxDrawdown: 8.5,
  concentration: 0.67,
  diversificationRatio: 0.78
})

// Diversification data
const diversificationData = ref({
  sectors: [
    { name: 'Layer 1', allocation: 68.5, target: 60, status: 'overweight' },
    { name: 'DeFi', allocation: 18.2, target: 25, status: 'underweight' },
    { name: 'Stablecoins', allocation: 12.0, target: 15, status: 'underweight' },
    { name: 'NFTs', allocation: 1.3, target: 0, status: 'target' }
  ],
  chains: [
    { name: 'Ethereum', allocation: 78.5 },
    { name: 'Polygon', allocation: 12.3 },
    { name: 'Arbitrum', allocation: 6.8 },
    { name: 'Optimism', allocation: 2.4 }
  ]
})

// Correlation data
const correlationData = ref([
  ['ETH', 1.00, 0.85, 0.12, 0.67, 0.72],
  ['BTC', 0.85, 1.00, 0.08, 0.54, 0.61],
  ['USDC', 0.12, 0.08, 1.00, 0.09, 0.11],
  ['UNI', 0.67, 0.54, 0.09, 1.00, 0.78],
  ['AAVE', 0.72, 0.61, 0.11, 0.78, 1.00]
])

// Yield data
const yieldData = ref({
  totalStaked: 450000,
  totalRewards: 18750,
  apr: 4.8,
  positions: [
    {
      protocol: 'Ethereum 2.0',
      asset: 'ETH',
      staked: 64.0,
      value: 210000,
      rewards: 8.2,
      apr: 4.2
    },
    {
      protocol: 'Uniswap V3',
      asset: 'ETH-USDC',
      staked: 150000,
      value: 150000,
      rewards: 750,
      apr: 6.8
    }
  ]
})

// Transaction history
const transactionHistory = ref([
  {
    id: '1',
    type: 'buy',
    asset: 'ETH',
    quantity: 5.0,
    price: 3250,
    value: 16250,
    fee: 25,
    timestamp: new Date(Date.now() - 3600000).toISOString(),
    status: 'completed',
    txHash: '0x1234...5678'
  },
  {
    id: '2',
    type: 'sell',
    asset: 'BTC',
    quantity: 0.5,
    price: 68000,
    value: 34000,
    fee: 50,
    timestamp: new Date(Date.now() - 7200000).toISOString(),
    status: 'completed',
    txHash: '0x9abc...def0'
  }
])

// Methods
function formatCurrency(amount: number): string {
  if (amount >= 1e9) return `${(amount / 1e9).toFixed(2)}B`
  if (amount >= 1e6) return `${(amount / 1e6).toFixed(2)}M`
  if (amount >= 1e3) return `${(amount / 1e3).toFixed(2)}K`
  return amount.toLocaleString()
}

function getChangeColor(change: number): string {
  return change >= 0 ? 'text-green-400' : 'text-red-400'
}

function getAssetColor(symbol: string): string {
  const colors: Record<string, string> = {
    ETH: '#627EEA',
    BTC: '#F7931A',
    USDC: '#2775CA',
    UNI: '#FF007A',
    AAVE: '#B6509E'
  }
  return colors[symbol] || '#8B5CF6'
}

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

function handlePositionAction(action: string, position: any) {
  notificationStore.notifySystem(
    `${action} Position`,
    `${action} ${position.symbol} position`,
    'info'
  )
}

function handleSortChange(sort: any) {
  console.log('Sort changed:', sort)
}

function handleRebalance() {
  notificationStore.notifySystem(
    'Rebalancing Portfolio',
    'Optimizing asset allocation...',
    'info'
  )
}

function handleStressTest() {
  notificationStore.notifySystem(
    'Running Stress Test',
    'Analyzing portfolio under extreme scenarios...',
    'info'
  )
}

function handleClaimRewards() {
  notificationStore.notifySystem(
    'Claiming Rewards',
    'Processing reward claims...',
    'info'
  )
}

function handleStakeAction(action: string, data: any) {
  notificationStore.notifySystem(
    `${action} Assets`,
    `Processing ${action.toLowerCase()} request...`,
    'info'
  )
}

function handleExportTransactions() {
  emit('data-export', { type: 'transactions', data: transactionHistory.value })
}

function handleFilterChange(filters: any) {
  console.log('Filters changed:', filters)
}
</script>