<template>
  <div class="space-y-6">
    <!-- Portfolio Header -->
    <Card variant="glass">
      <div class="flex items-center justify-between p-6">
        <div class="flex items-center space-x-4">
          <div class="w-12 h-12 bg-gradient-to-br from-moby-400 to-moby-600 rounded-xl flex items-center justify-center">
            <HeroIcon name="BriefcaseIcon" class="w-6 h-6 text-white" />
          </div>
          <div>
            <h1 class="text-2xl font-bold text-white">{{ portfolio.name }}</h1>
            <p class="text-sm text-white/60">{{ portfolio.description }}</p>
          </div>
        </div>

        <div class="flex items-center space-x-4">
          <div class="text-right">
            <div class="text-3xl font-bold text-white">${{ formatAmount(portfolio.totalValue) }}</div>
            <div :class="['text-sm flex items-center space-x-1', getChangeColor(portfolio.change24h)]">
              <HeroIcon :name="portfolio.change24h >= 0 ? 'ArrowTrendingUpIcon' : 'ArrowTrendingDownIcon'" class="w-4 h-4" />
              <span>{{ portfolio.change24h >= 0 ? '+' : '' }}{{ portfolio.change24h.toFixed(2) }}%</span>
              <span class="text-white/60">(24h)</span>
            </div>
          </div>

          <div class="flex space-x-2">
            <Button
              variant="primary"
              icon-left="PlusIcon"
              @click="$emit('add-position')"
            >
              Add Position
            </Button>
            <Button
              variant="secondary"
              icon-left="ArrowPathIcon"
              @click="$emit('rebalance')"
            >
              Rebalance
            </Button>
            <Button
              variant="ghost"
              icon-left="Cog6ToothIcon"
              @click="$emit('settings')"
            />
          </div>
        </div>
      </div>
    </Card>

    <!-- Quick Stats -->
    <div class="grid grid-cols-2 lg:grid-cols-5 gap-4">
      <Card variant="glass" class="p-4">
        <div class="flex items-center space-x-2 mb-2">
          <HeroIcon name="CurrencyDollarIcon" class="w-4 h-4 text-green-400" />
          <span class="text-xs text-white/60">Total P&L</span>
        </div>
        <div :class="['text-xl font-bold', getChangeColor(portfolio.totalPnL)]">
          {{ portfolio.totalPnL >= 0 ? '+' : '' }}${{ formatAmount(Math.abs(portfolio.totalPnL)) }}
        </div>
        <div class="text-xs text-white/60">All time</div>
      </Card>

      <Card variant="glass" class="p-4">
        <div class="flex items-center space-x-2 mb-2">
          <HeroIcon name="ChartBarIcon" class="w-4 h-4 text-blue-400" />
          <span class="text-xs text-white/60">Diversity Score</span>
        </div>
        <div :class="['text-xl font-bold', getDiversityColor(portfolio.diversityScore)]">
          {{ portfolio.diversityScore }}/100
        </div>
        <div class="text-xs text-white/60">{{ getDiversityLabel(portfolio.diversityScore) }}</div>
      </Card>

      <Card variant="glass" class="p-4">
        <div class="flex items-center space-x-2 mb-2">
          <HeroIcon name="ShieldCheckIcon" class="w-4 h-4 text-purple-400" />
          <span class="text-xs text-white/60">Risk Level</span>
        </div>
        <div :class="['text-xl font-bold', getRiskColor(portfolio.riskLevel)]">
          {{ portfolio.riskLevel }}
        </div>
        <div class="text-xs text-white/60">Current assessment</div>
      </Card>

      <Card variant="glass" class="p-4">
        <div class="flex items-center space-x-2 mb-2">
          <HeroIcon name="ClockIcon" class="w-4 h-4 text-yellow-400" />
          <span class="text-xs text-white/60">Last Activity</span>
        </div>
        <div class="text-xl font-bold text-white">{{ portfolio.lastActivity }}</div>
        <div class="text-xs text-white/60">{{ formatTimeAgo(portfolio.lastActivityTime) }}</div>
      </Card>

      <Card variant="glass" class="p-4">
        <div class="flex items-center space-x-2 mb-2">
          <HeroIcon name="BanknotesIcon" class="w-4 h-4 text-orange-400" />
          <span class="text-xs text-white/60">Available Cash</span>
        </div>
        <div class="text-xl font-bold text-white">${{ formatAmount(portfolio.availableCash) }}</div>
        <div class="text-xs text-white/60">Ready to invest</div>
      </Card>
    </div>

    <!-- Portfolio Allocation Chart -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <Card variant="glass">
        <template #header>
          <div class="flex items-center justify-between w-full">
            <h3 class="text-lg font-semibold text-white">Asset Allocation</h3>
            <Button
              variant="ghost"
              size="xs"
              icon-right="ChevronRightIcon"
              @click="$emit('view-allocation')"
            >
              Details
            </Button>
          </div>
        </template>

        <div class="p-6">
          <div class="relative w-48 h-48 mx-auto mb-6">
            <svg class="w-48 h-48" viewBox="0 0 200 200">
              <g v-for="(segment, index) in allocationSegments" :key="index">
                <path
                  :d="segment.path"
                  :fill="segment.color"
                  class="opacity-80 hover:opacity-100 transition-opacity cursor-pointer"
                  @mouseenter="showAllocationTooltip(segment)"
                  @mouseleave="hideTooltip"
                />
              </g>
              <!-- Center text -->
              <text x="100" y="95" text-anchor="middle" class="text-xs fill-white/60">Total Assets</text>
              <text x="100" y="110" text-anchor="middle" class="text-sm font-bold fill-white">
                {{ portfolio.positions.length }}
              </text>
            </svg>
          </div>

          <div class="space-y-2">
            <div
              v-for="asset in topAssets"
              :key="asset.symbol"
              class="flex items-center justify-between"
            >
              <div class="flex items-center space-x-2">
                <div :class="asset.colorClass" class="w-3 h-3 rounded-full"></div>
                <img :src="getAssetIcon(asset.symbol)" :alt="asset.symbol" class="w-4 h-4 rounded-full" />
                <span class="text-sm text-white/70">{{ asset.symbol }}</span>
              </div>
              <div class="text-sm font-medium text-white">{{ asset.percentage.toFixed(1) }}%</div>
            </div>
            <div v-if="portfolio.positions.length > 5" class="flex items-center justify-between">
              <div class="flex items-center space-x-2">
                <div class="w-3 h-3 rounded-full bg-gray-400"></div>
                <span class="text-sm text-white/70">Others</span>
              </div>
              <div class="text-sm font-medium text-white">
                {{ (100 - topAssets.reduce((sum, a) => sum + a.percentage, 0)).toFixed(1) }}%
              </div>
            </div>
          </div>
        </div>
      </Card>

      <!-- Performance Summary -->
      <Card variant="glass">
        <template #header>
          <div class="flex items-center justify-between w-full">
            <h3 class="text-lg font-semibold text-white">Performance Summary</h3>
            <select
              v-model="performancePeriod"
              class="bg-slate-800/50 border border-slate-600/50 rounded-lg px-3 py-1 text-white text-xs focus:outline-none focus:border-moby-500/50"
            >
              <option value="24h">24 Hours</option>
              <option value="7d">7 Days</option>
              <option value="30d">30 Days</option>
              <option value="90d">90 Days</option>
              <option value="1y">1 Year</option>
            </select>
          </div>
        </template>

        <div class="p-6 space-y-4">
          <div class="grid grid-cols-2 gap-4">
            <div class="bg-slate-800/30 rounded-lg p-3">
              <div class="text-xs text-white/60 mb-1">Return</div>
              <div :class="['text-lg font-bold', getChangeColor(performanceMetrics.return)]">
                {{ performanceMetrics.return >= 0 ? '+' : '' }}{{ performanceMetrics.return.toFixed(2) }}%
              </div>
            </div>
            <div class="bg-slate-800/30 rounded-lg p-3">
              <div class="text-xs text-white/60 mb-1">Volatility</div>
              <div :class="['text-lg font-bold', getVolatilityColor(performanceMetrics.volatility)]">
                {{ performanceMetrics.volatility.toFixed(1) }}%
              </div>
            </div>
            <div class="bg-slate-800/30 rounded-lg p-3">
              <div class="text-xs text-white/60 mb-1">Sharpe Ratio</div>
              <div :class="['text-lg font-bold', getSharpeColor(performanceMetrics.sharpe)]">
                {{ performanceMetrics.sharpe.toFixed(2) }}
              </div>
            </div>
            <div class="bg-slate-800/30 rounded-lg p-3">
              <div class="text-xs text-white/60 mb-1">Max Drawdown</div>
              <div class="text-lg font-bold text-red-400">
                {{ performanceMetrics.maxDrawdown.toFixed(1) }}%
              </div>
            </div>
          </div>

          <!-- Mini Performance Chart -->
          <div class="h-24 bg-slate-800/20 rounded-lg p-2">
            <svg class="w-full h-full" viewBox="0 0 300 80">
              <path
                :d="performanceChartPath"
                fill="none"
                stroke="#60a5fa"
                stroke-width="2"
                class="drop-shadow-sm"
              />
              <path
                :d="performanceAreaPath"
                fill="url(#performanceGradient)"
                class="opacity-30"
              />
              <defs>
                <linearGradient id="performanceGradient" x1="0%" y1="0%" x2="0%" y2="100%">
                  <stop offset="0%" style="stop-color:#60a5fa;stop-opacity:0.6" />
                  <stop offset="100%" style="stop-color:#60a5fa;stop-opacity:0" />
                </linearGradient>
              </defs>
            </svg>
          </div>
        </div>
      </Card>
    </div>

    <!-- Top Holdings -->
    <Card variant="glass">
      <template #header>
        <div class="flex items-center justify-between w-full">
          <h3 class="text-lg font-semibold text-white">Top Holdings</h3>
          <Button
            variant="ghost"
            size="xs"
            icon-right="ChevronRightIcon"
            @click="$emit('view-all-positions')"
          >
            View All
          </Button>
        </div>
      </template>

      <div class="p-6">
        <div class="space-y-3">
          <div
            v-for="position in topHoldings"
            :key="position.symbol"
            class="flex items-center space-x-4 p-3 bg-slate-800/30 hover:bg-slate-700/40 rounded-lg transition-all cursor-pointer"
            @click="$emit('view-position', position)"
          >
            <!-- Asset Info -->
            <div class="flex items-center space-x-3 flex-1">
              <img
                :src="getAssetIcon(position.symbol)"
                :alt="position.symbol"
                class="w-8 h-8 rounded-full"
              />
              <div>
                <div class="text-sm font-medium text-white">{{ position.symbol }}</div>
                <div class="text-xs text-white/60">{{ position.name }}</div>
              </div>
            </div>

            <!-- Holdings -->
            <div class="text-right">
              <div class="text-sm font-bold text-white">{{ formatAmount(position.amount) }}</div>
              <div class="text-xs text-white/60">${{ formatAmount(position.value) }}</div>
            </div>

            <!-- Allocation -->
            <div class="text-right min-w-16">
              <div class="text-sm font-bold text-white">{{ position.allocation.toFixed(1) }}%</div>
              <div class="w-12 h-2 bg-slate-700/50 rounded-full overflow-hidden mt-1">
                <div
                  class="h-full bg-blue-400 transition-all duration-500"
                  :style="{ width: `${position.allocation}%` }"
                ></div>
              </div>
            </div>

            <!-- P&L -->
            <div class="text-right min-w-20">
              <div :class="['text-sm font-bold', getChangeColor(position.pnl)]">
                {{ position.pnl >= 0 ? '+' : '' }}${{ formatAmount(Math.abs(position.pnl)) }}
              </div>
              <div :class="['text-xs', getChangeColor(position.pnlPercent)]">
                {{ position.pnlPercent >= 0 ? '+' : '' }}{{ position.pnlPercent.toFixed(1) }}%
              </div>
            </div>

            <HeroIcon name="ChevronRightIcon" class="w-4 h-4 text-white/40" />
          </div>
        </div>
      </div>
    </Card>

    <!-- Recent Transactions -->
    <Card variant="glass">
      <template #header>
        <div class="flex items-center justify-between w-full">
          <h3 class="text-lg font-semibold text-white">Recent Activity</h3>
          <Button
            variant="ghost"
            size="xs"
            icon-right="ChevronRightIcon"
            @click="$emit('view-all-transactions')"
          >
            View All
          </Button>
        </div>
      </template>

      <div class="p-6">
        <div class="space-y-3">
          <div
            v-for="transaction in recentTransactions"
            :key="transaction.id"
            class="flex items-center space-x-4 p-3 bg-slate-800/30 rounded-lg"
          >
            <!-- Transaction Type -->
            <div :class="getTransactionIconClass(transaction.type)" class="p-2 rounded-lg">
              <HeroIcon :name="getTransactionIcon(transaction.type)" class="w-4 h-4" />
            </div>

            <!-- Transaction Info -->
            <div class="flex-1">
              <div class="flex items-center space-x-2">
                <span class="text-sm font-medium text-white">{{ getTransactionLabel(transaction.type) }}</span>
                <img :src="getAssetIcon(transaction.asset)" :alt="transaction.asset" class="w-4 h-4 rounded-full" />
                <span class="text-sm text-white/70">{{ transaction.asset }}</span>
              </div>
              <div class="text-xs text-white/60">{{ formatTimeAgo(transaction.timestamp) }}</div>
            </div>

            <!-- Amount -->
            <div class="text-right">
              <div class="text-sm font-bold text-white">
                {{ formatAmount(transaction.amount) }} {{ transaction.asset }}
              </div>
              <div class="text-xs text-white/60">${{ formatAmount(transaction.usdValue) }}</div>
            </div>

            <!-- Status -->
            <div :class="getTransactionStatusClass(transaction.status)" class="px-2 py-1 rounded text-xs font-medium">
              {{ transaction.status }}
            </div>
          </div>
        </div>
      </div>
    </Card>

    <!-- Allocation Tooltip -->
    <div
      v-if="tooltip.show"
      :style="{ left: tooltip.x + 'px', top: tooltip.y + 'px' }"
      class="fixed z-10 bg-slate-800/90 backdrop-blur border border-white/20 rounded-lg p-3 text-xs pointer-events-none"
    >
      <div class="font-semibold text-white">{{ tooltip.label }}</div>
      <div class="text-white/70">${{ formatAmount(tooltip.value) }}</div>
      <div class="text-white/60">{{ tooltip.percentage }}% of portfolio</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

import Card from '@components/ui/Card.vue'
import Button from '@components/ui/Button.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'

interface Position {
  symbol: string
  name: string
  amount: number
  value: number
  allocation: number
  pnl: number
  pnlPercent: number
  price: number
}

interface Transaction {
  id: string
  type: 'buy' | 'sell' | 'swap' | 'transfer'
  asset: string
  amount: number
  usdValue: number
  timestamp: string
  status: 'completed' | 'pending' | 'failed'
}

interface Portfolio {
  name: string
  description: string
  totalValue: number
  change24h: number
  totalPnL: number
  diversityScore: number
  riskLevel: string
  lastActivity: string
  lastActivityTime: string
  availableCash: number
  positions: Position[]
  transactions: Transaction[]
}

interface Props {
  portfolio: Portfolio
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  portfolio: () => ({
    name: 'Main Portfolio',
    description: 'Primary trading portfolio',
    totalValue: 125485.67,
    change24h: 3.45,
    totalPnL: 28450.32,
    diversityScore: 78,
    riskLevel: 'Moderate',
    lastActivity: 'ETH Buy',
    lastActivityTime: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
    availableCash: 15230.50,
    positions: [],
    transactions: []
  })
})

const emit = defineEmits<{
  'add-position': []
  'rebalance': []
  'settings': []
  'view-allocation': []
  'view-all-positions': []
  'view-position': [position: Position]
  'view-all-transactions': []
}>()

const performancePeriod = ref('30d')

const tooltip = ref({
  show: false,
  x: 0,
  y: 0,
  label: '',
  value: 0,
  percentage: 0
})

// Generate mock data for demonstration
const generateMockPositions = (): Position[] => {
  const assets = [
    { symbol: 'ETH', name: 'Ethereum' },
    { symbol: 'BTC', name: 'Bitcoin' },
    { symbol: 'UNI', name: 'Uniswap' },
    { symbol: 'AAVE', name: 'Aave' },
    { symbol: 'COMP', name: 'Compound' },
    { symbol: 'SUSHI', name: 'SushiSwap' },
    { symbol: 'CRV', name: 'Curve' },
    { symbol: 'MKR', name: 'Maker' }
  ]

  return assets.map((asset, index) => {
    const allocation = index === 0 ? 35 : index === 1 ? 25 : 40 / (assets.length - 2)
    const value = (props.portfolio.totalValue * allocation) / 100
    const price = 1000 + Math.random() * 2000
    const amount = value / price
    const pnl = (Math.random() - 0.3) * value * 0.5
    const pnlPercent = (pnl / (value - pnl)) * 100

    return {
      symbol: asset.symbol,
      name: asset.name,
      amount,
      value,
      allocation,
      pnl,
      pnlPercent,
      price
    }
  })
}

const generateMockTransactions = (): Transaction[] => {
  const types: Transaction['type'][] = ['buy', 'sell', 'swap', 'transfer']
  const statuses: Transaction['status'][] = ['completed', 'pending', 'failed']
  const assets = ['ETH', 'BTC', 'UNI', 'AAVE', 'COMP']

  return Array.from({ length: 10 }, (_, i) => ({
    id: `tx-${i}`,
    type: types[Math.floor(Math.random() * types.length)],
    asset: assets[Math.floor(Math.random() * assets.length)],
    amount: Math.random() * 100,
    usdValue: 100 + Math.random() * 10000,
    timestamp: new Date(Date.now() - Math.random() * 7 * 24 * 60 * 60 * 1000).toISOString(),
    status: i < 8 ? 'completed' : statuses[Math.floor(Math.random() * statuses.length)]
  }))
}

const positions = computed(() => props.portfolio.positions.length ? props.portfolio.positions : generateMockPositions())
const transactions = computed(() => props.portfolio.transactions.length ? props.portfolio.transactions : generateMockTransactions())

const topHoldings = computed(() => positions.value.slice(0, 5))
const recentTransactions = computed(() => transactions.value.slice(0, 5))

const topAssets = computed(() => {
  return positions.value
    .sort((a, b) => b.allocation - a.allocation)
    .slice(0, 5)
    .map((position, index) => ({
      ...position,
      colorClass: getAssetColorClass(index),
      percentage: position.allocation
    }))
})

// Allocation pie chart segments
const allocationSegments = computed(() => {
  let currentAngle = 0
  const radius = 80
  const centerX = 100
  const centerY = 100

  return topAssets.value.map((asset, index) => {
    const angle = (asset.percentage / 100) * 2 * Math.PI
    const startAngle = currentAngle
    const endAngle = currentAngle + angle

    const x1 = centerX + radius * Math.cos(startAngle)
    const y1 = centerY + radius * Math.sin(startAngle)
    const x2 = centerX + radius * Math.cos(endAngle)
    const y2 = centerY + radius * Math.sin(endAngle)

    const largeArcFlag = angle > Math.PI ? 1 : 0

    const path = [
      `M ${centerX} ${centerY}`,
      `L ${x1} ${y1}`,
      `A ${radius} ${radius} 0 ${largeArcFlag} 1 ${x2} ${y2}`,
      `Z`
    ].join(' ')

    currentAngle += angle

    return {
      path,
      color: getAssetColorHex(index),
      symbol: asset.symbol,
      value: asset.value,
      percentage: asset.percentage
    }
  })
})

// Performance metrics
const performanceMetrics = computed(() => {
  const periods: Record<string, number> = {
    '24h': 1,
    '7d': 7,
    '30d': 30,
    '90d': 90,
    '1y': 365
  }

  const days = periods[performancePeriod.value] || 30

  // Generate mock performance data
  const returns = Array.from({ length: days }, () => (Math.random() - 0.48) * 2)
  const avgReturn = returns.reduce((sum, r) => sum + r, 0) / returns.length
  const variance = returns.reduce((sum, r) => sum + Math.pow(r - avgReturn, 2), 0) / returns.length
  const volatility = Math.sqrt(variance) * Math.sqrt(252) // Annualized

  return {
    return: avgReturn * days,
    volatility,
    sharpe: volatility === 0 ? 0 : (avgReturn * 252) / volatility,
    maxDrawdown: Math.min(...returns.map((_, i) => returns.slice(0, i + 1).reduce((sum, r) => sum + r, 0))) * -1
  }
})

// Performance chart data
const performanceChartPath = computed(() => {
  const points = Array.from({ length: 30 }, (_, i) => {
    const x = (i / 29) * 290 + 5
    const y = 40 + Math.sin(i / 5) * 15 + (Math.random() - 0.5) * 10
    return `${i === 0 ? 'M' : 'L'} ${x} ${y}`
  })
  return points.join(' ')
})

const performanceAreaPath = computed(() => {
  const linePath = performanceChartPath.value
  const points = linePath.split(' ')
  const lastPoint = points[points.length - 2] + ' ' + points[points.length - 1]
  const firstPoint = points[1] + ' ' + points[2]

  return `${linePath} L ${lastPoint.split(' ')[0]} 70 L ${firstPoint.split(' ')[0]} 70 Z`
})

// Methods
function formatAmount(amount: number): string {
  if (amount >= 1e9) return `${(amount / 1e9).toFixed(2)}B`
  if (amount >= 1e6) return `${(amount / 1e6).toFixed(2)}M`
  if (amount >= 1e3) return `${(amount / 1e3).toFixed(2)}K`
  return amount.toFixed(2)
}

function formatTimeAgo(timestamp: string): string {
  const date = new Date(timestamp)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / (1000 * 60))

  if (diffMins < 1) return 'Just now'
  if (diffMins < 60) return `${diffMins}m ago`
  if (diffMins < 1440) return `${Math.floor(diffMins / 60)}h ago`
  return date.toLocaleDateString()
}

function getChangeColor(change: number): string {
  return change >= 0 ? 'text-green-400' : 'text-red-400'
}

function getDiversityColor(score: number): string {
  if (score >= 80) return 'text-green-400'
  if (score >= 60) return 'text-yellow-400'
  return 'text-red-400'
}

function getDiversityLabel(score: number): string {
  if (score >= 80) return 'Well diversified'
  if (score >= 60) return 'Moderately diversified'
  return 'Concentrated'
}

function getRiskColor(risk: string): string {
  switch (risk.toLowerCase()) {
    case 'low': return 'text-green-400'
    case 'moderate': return 'text-yellow-400'
    case 'high': return 'text-red-400'
    default: return 'text-gray-400'
  }
}

function getVolatilityColor(volatility: number): string {
  if (volatility <= 20) return 'text-green-400'
  if (volatility <= 40) return 'text-yellow-400'
  return 'text-red-400'
}

function getSharpeColor(sharpe: number): string {
  if (sharpe >= 1.5) return 'text-green-400'
  if (sharpe >= 1) return 'text-yellow-400'
  return 'text-red-400'
}

function getAssetIcon(symbol: string): string {
  const iconMap: Record<string, string> = {
    ETH: '/tokens/eth.svg',
    BTC: '/tokens/btc.svg',
    UNI: '/tokens/uni.svg',
    AAVE: '/tokens/aave.svg',
    COMP: '/tokens/comp.svg',
    SUSHI: '/tokens/sushi.svg',
    CRV: '/tokens/crv.svg',
    MKR: '/tokens/mkr.svg'
  }
  return iconMap[symbol] || '/tokens/default.svg'
}

function getAssetColorClass(index: number): string {
  const colors = [
    'bg-blue-400',
    'bg-purple-400',
    'bg-green-400',
    'bg-yellow-400',
    'bg-red-400',
    'bg-pink-400',
    'bg-indigo-400',
    'bg-cyan-400'
  ]
  return colors[index % colors.length]
}

function getAssetColorHex(index: number): string {
  const colors = [
    '#60a5fa',
    '#a855f7',
    '#4ade80',
    '#facc15',
    '#f87171',
    '#f472b6',
    '#6366f1',
    '#06b6d4'
  ]
  return colors[index % colors.length]
}

function getTransactionIcon(type: string): string {
  switch (type) {
    case 'buy': return 'ArrowDownIcon'
    case 'sell': return 'ArrowUpIcon'
    case 'swap': return 'ArrowsRightLeftIcon'
    case 'transfer': return 'PaperAirplaneIcon'
    default: return 'DocumentIcon'
  }
}

function getTransactionIconClass(type: string): string {
  switch (type) {
    case 'buy': return 'bg-green-500/20 text-green-400'
    case 'sell': return 'bg-red-500/20 text-red-400'
    case 'swap': return 'bg-blue-500/20 text-blue-400'
    case 'transfer': return 'bg-purple-500/20 text-purple-400'
    default: return 'bg-gray-500/20 text-gray-400'
  }
}

function getTransactionLabel(type: string): string {
  switch (type) {
    case 'buy': return 'Buy'
    case 'sell': return 'Sell'
    case 'swap': return 'Swap'
    case 'transfer': return 'Transfer'
    default: return 'Unknown'
  }
}

function getTransactionStatusClass(status: string): string {
  switch (status) {
    case 'completed': return 'bg-green-500/20 text-green-400'
    case 'pending': return 'bg-yellow-500/20 text-yellow-400'
    case 'failed': return 'bg-red-500/20 text-red-400'
    default: return 'bg-gray-500/20 text-gray-400'
  }
}

function showAllocationTooltip(segment: any) {
  tooltip.value = {
    show: true,
    x: 300,
    y: 200,
    label: segment.symbol,
    value: segment.value,
    percentage: segment.percentage.toFixed(1)
  }
}

function hideTooltip() {
  tooltip.value.show = false
}
</script>

<style scoped>
/* Chart animations */
path {
  transition: all 0.3s ease;
}

/* Card hover effects */
.hover\:bg-slate-700\/40:hover {
  background-color: rgba(51, 65, 85, 0.4);
}

/* Allocation bar animation */
.transition-all {
  transition: all 0.5s ease;
}
</style>