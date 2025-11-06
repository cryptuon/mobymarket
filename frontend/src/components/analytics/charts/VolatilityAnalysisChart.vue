<template>
  <Card variant="glass">
    <template #header>
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center space-x-3">
          <HeroIcon name="ChartBarIcon" class="w-5 h-5 text-red-400" />
          <div>
            <h3 class="text-lg font-semibold text-white">Volatility Analysis</h3>
            <p class="text-xs text-white/60">Price volatility and risk metrics</p>
          </div>
        </div>

        <div class="flex items-center space-x-2">
          <select
            v-model="selectedPeriod"
            class="bg-slate-800/50 border border-slate-600/50 rounded-lg px-3 py-1 text-white text-xs focus:outline-none focus:border-moby-500/50"
          >
            <option value="7D">7 Days</option>
            <option value="30D">30 Days</option>
            <option value="90D">90 Days</option>
            <option value="1Y">1 Year</option>
          </select>

          <div class="flex bg-slate-800/50 rounded-lg p-1">
            <button
              v-for="metric in volatilityMetrics"
              :key="metric.value"
              @click="selectedMetric = metric.value"
              :class="[
                'px-2 py-1 text-xs rounded transition-all',
                selectedMetric === metric.value
                  ? 'bg-moby-500 text-white'
                  : 'text-white/60 hover:text-white'
              ]"
            >
              {{ metric.label }}
            </button>
          </div>
        </div>
      </div>
    </template>

    <div class="space-y-6">
      <!-- Loading State -->
      <div v-if="loading" class="space-y-4">
        <div class="animate-pulse">
          <div class="h-64 bg-slate-700/30 rounded-lg"></div>
        </div>
        <div class="grid grid-cols-3 gap-4">
          <div v-for="i in 3" :key="i" class="animate-pulse">
            <div class="h-20 bg-slate-700/30 rounded-lg"></div>
          </div>
        </div>
      </div>

      <!-- Volatility Chart -->
      <div v-else class="relative">
        <div class="h-64 bg-slate-800/20 rounded-lg p-4">
          <svg class="w-full h-full" viewBox="0 0 800 240">
            <!-- Grid Lines -->
            <defs>
              <pattern id="volatilityGrid" width="40" height="24" patternUnits="userSpaceOnUse">
                <path d="M 40 0 L 0 0 0 24" fill="none" stroke="rgba(255,255,255,0.1)" stroke-width="0.5"/>
              </pattern>
            </defs>
            <rect width="100%" height="100%" fill="url(#volatilityGrid)" />

            <!-- Y-axis labels -->
            <g class="text-xs fill-white/60">
              <text x="10" y="20">{{ maxVolatility.toFixed(1) }}%</text>
              <text x="10" y="80">{{ (maxVolatility * 0.75).toFixed(1) }}%</text>
              <text x="10" y="140">{{ (maxVolatility * 0.5).toFixed(1) }}%</text>
              <text x="10" y="200">{{ (maxVolatility * 0.25).toFixed(1) }}%</text>
              <text x="10" y="235">0%</text>
            </g>

            <!-- Volatility Area Chart -->
            <path
              :d="volatilityAreaPath"
              :fill="getVolatilityGradient()"
              class="opacity-40"
            />

            <!-- Volatility Line -->
            <path
              :d="volatilityLinePath"
              fill="none"
              :stroke="getVolatilityColor(currentVolatility)"
              stroke-width="2"
              class="drop-shadow-sm"
            />

            <!-- Volatility threshold lines -->
            <line
              x1="40"
              x2="760"
              :y1="thresholdY.high"
              :y2="thresholdY.high"
              stroke="#ef4444"
              stroke-width="1"
              stroke-dasharray="5,5"
              class="opacity-60"
            />
            <line
              x1="40"
              x2="760"
              :y1="thresholdY.low"
              :y2="thresholdY.low"
              stroke="#10b981"
              stroke-width="1"
              stroke-dasharray="5,5"
              class="opacity-60"
            />

            <!-- Data Points -->
            <g v-for="(point, index) in volatilityPoints" :key="index">
              <circle
                :cx="point.x"
                :cy="point.y"
                r="3"
                :fill="getVolatilityColor(point.value)"
                class="opacity-0 hover:opacity-100 transition-opacity cursor-pointer"
                @mouseenter="showVolatilityTooltip(point, index)"
                @mouseleave="hideTooltip"
              />
            </g>
          </svg>
        </div>

        <!-- Volatility Indicators -->
        <div class="flex items-center justify-center space-x-6 mt-4">
          <div class="flex items-center space-x-2">
            <div class="w-3 h-0.5 bg-red-400 border-dashed border-t border-red-400"></div>
            <span class="text-xs text-white/70">High Risk ({{ highThreshold }}%)</span>
          </div>
          <div class="flex items-center space-x-2">
            <div class="w-3 h-0.5 bg-green-400 border-dashed border-t border-green-400"></div>
            <span class="text-xs text-white/70">Low Risk ({{ lowThreshold }}%)</span>
          </div>
        </div>

        <!-- Tooltip -->
        <div
          v-if="tooltip.show"
          :style="{ left: tooltip.x + 'px', top: tooltip.y + 'px' }"
          class="absolute z-10 bg-slate-800/90 backdrop-blur border border-white/20 rounded-lg p-3 text-xs pointer-events-none"
        >
          <div class="font-semibold text-white">{{ selectedMetric.toUpperCase() }} Volatility</div>
          <div class="text-white/70">{{ tooltip.date }}</div>
          <div :class="['font-bold', getVolatilityColor(tooltip.value)]">
            {{ tooltip.value.toFixed(2) }}%
          </div>
          <div class="text-white/60 mt-1">{{ getVolatilityLabel(tooltip.value) }}</div>
        </div>
      </div>

      <!-- Volatility Statistics -->
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <div class="bg-slate-800/30 rounded-lg p-4 text-center">
          <div class="flex items-center justify-center space-x-2 mb-2">
            <HeroIcon name="ChartBarIcon" class="w-4 h-4 text-red-400" />
            <span class="text-xs text-white/60">Current Volatility</span>
          </div>
          <div :class="['text-xl font-bold', getVolatilityColor(currentVolatility)]">
            {{ currentVolatility.toFixed(1) }}%
          </div>
          <div class="text-xs text-white/60">{{ getVolatilityLabel(currentVolatility) }}</div>
        </div>

        <div class="bg-slate-800/30 rounded-lg p-4 text-center">
          <div class="flex items-center justify-center space-x-2 mb-2">
            <HeroIcon name="ArrowTrendingUpIcon" class="w-4 h-4 text-orange-400" />
            <span class="text-xs text-white/60">30-Day Avg</span>
          </div>
          <div :class="['text-xl font-bold', getVolatilityColor(avgVolatility)]">
            {{ avgVolatility.toFixed(1) }}%
          </div>
          <div :class="['text-xs', getChangeColor(volatilityChange)]">
            {{ volatilityChange >= 0 ? '+' : '' }}{{ volatilityChange.toFixed(1) }}%
          </div>
        </div>

        <div class="bg-slate-800/30 rounded-lg p-4 text-center">
          <div class="flex items-center justify-center space-x-2 mb-2">
            <HeroIcon name="ExclamationTriangleIcon" class="w-4 h-4 text-yellow-400" />
            <span class="text-xs text-white/60">Max Volatility</span>
          </div>
          <div class="text-xl font-bold text-red-400">{{ maxVolatility.toFixed(1) }}%</div>
          <div class="text-xs text-white/60">{{ selectedPeriod }} peak</div>
        </div>

        <div class="bg-slate-800/30 rounded-lg p-4 text-center">
          <div class="flex items-center justify-center space-x-2 mb-2">
            <HeroIcon name="ShieldCheckIcon" class="w-4 h-4 text-green-400" />
            <span class="text-xs text-white/60">Min Volatility</span>
          </div>
          <div class="text-xl font-bold text-green-400">{{ minVolatility.toFixed(1) }}%</div>
          <div class="text-xs text-white/60">{{ selectedPeriod }} low</div>
        </div>
      </div>

      <!-- Volatility Breakdown by Asset -->
      <div class="space-y-4">
        <h4 class="text-sm font-semibold text-white">Asset Volatility Breakdown</h4>
        <div class="space-y-3">
          <div
            v-for="asset in assetVolatility"
            :key="asset.symbol"
            class="flex items-center space-x-4"
          >
            <div class="flex items-center space-x-2 min-w-0 flex-1">
              <img
                :src="getAssetIcon(asset.symbol)"
                :alt="asset.symbol"
                class="w-6 h-6 rounded-full"
              />
              <span class="text-sm font-medium text-white">{{ asset.symbol }}</span>
              <span class="text-xs text-white/60">{{ asset.name }}</span>
            </div>

            <div class="flex items-center space-x-3">
              <!-- Volatility Bar -->
              <div class="w-24 h-2 bg-slate-700/50 rounded-full overflow-hidden">
                <div
                  :class="getVolatilityBarClass(asset.volatility)"
                  :style="{ width: `${Math.min(100, (asset.volatility / maxVolatility) * 100)}%` }"
                  class="h-full transition-all duration-500"
                ></div>
              </div>

              <!-- Volatility Value -->
              <div class="text-right min-w-16">
                <div :class="['text-sm font-bold', getVolatilityColor(asset.volatility)]">
                  {{ asset.volatility.toFixed(1) }}%
                </div>
                <div class="text-xs text-white/60">{{ asset.allocation.toFixed(1) }}% weight</div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Volatility Regimes -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <!-- Volatility Distribution -->
        <div class="bg-slate-800/20 rounded-lg p-4">
          <h4 class="text-sm font-semibold text-white mb-3">Volatility Distribution</h4>
          <div class="space-y-2">
            <div
              v-for="regime in volatilityRegimes"
              :key="regime.label"
              class="flex items-center justify-between"
            >
              <div class="flex items-center space-x-2">
                <div :class="regime.colorClass" class="w-3 h-3 rounded-full"></div>
                <span class="text-sm text-white/70">{{ regime.label }}</span>
              </div>
              <div class="flex items-center space-x-2">
                <div class="w-20 h-2 bg-slate-700/50 rounded-full overflow-hidden">
                  <div
                    :class="regime.colorClass"
                    :style="{ width: `${regime.percentage}%` }"
                    class="h-full transition-all duration-500"
                  ></div>
                </div>
                <span class="text-sm font-medium text-white w-8 text-right">
                  {{ regime.percentage }}%
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- Risk Insights -->
        <div class="bg-slate-800/20 rounded-lg p-4">
          <h4 class="text-sm font-semibold text-white mb-3 flex items-center space-x-2">
            <HeroIcon name="LightBulbIcon" class="w-4 h-4 text-yellow-400" />
            <span>Risk Insights</span>
          </h4>
          <div class="space-y-2 text-sm text-white/70">
            <p v-for="insight in riskInsights" :key="insight" class="flex items-start space-x-2">
              <HeroIcon name="ChevronRightIcon" class="w-3 h-3 mt-0.5 text-moby-400 flex-shrink-0" />
              <span>{{ insight }}</span>
            </p>
          </div>
        </div>
      </div>

      <!-- Volatility Alerts -->
      <div v-if="volatilityAlerts.length > 0" class="space-y-2">
        <h4 class="text-sm font-semibold text-white flex items-center space-x-2">
          <HeroIcon name="BellIcon" class="w-4 h-4 text-yellow-400" />
          <span>Volatility Alerts</span>
        </h4>
        <div class="space-y-2">
          <div
            v-for="alert in volatilityAlerts"
            :key="alert.id"
            :class="getAlertClass(alert.severity)"
            class="p-3 rounded-lg border"
          >
            <div class="flex items-start space-x-2">
              <HeroIcon :name="getAlertIcon(alert.severity)" class="w-4 h-4 mt-0.5 flex-shrink-0" />
              <div class="flex-1 min-w-0">
                <div class="text-sm font-medium">{{ alert.title }}</div>
                <div class="text-xs mt-1 opacity-80">{{ alert.message }}</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

import Card from '@components/ui/Card.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'

interface VolatilityData {
  date: string
  volatility: number
  assets: Record<string, { volatility: number, allocation: number, name: string }>
}

interface VolatilityAlert {
  id: string
  title: string
  message: string
  severity: 'low' | 'medium' | 'high'
}

interface Props {
  data: VolatilityData[]
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

const selectedPeriod = ref('30D')
const selectedMetric = ref('realized')

const volatilityMetrics = [
  { label: 'Realized', value: 'realized' },
  { label: 'Implied', value: 'implied' },
  { label: 'GARCH', value: 'garch' }
]

const tooltip = ref({
  show: false,
  x: 0,
  y: 0,
  date: '',
  value: 0
})

const highThreshold = 40
const lowThreshold = 15

// Generate mock data
const generateMockVolatilityData = (): VolatilityData[] => {
  const data: VolatilityData[] = []
  const periods: Record<string, number> = {
    '7D': 7,
    '30D': 30,
    '90D': 90,
    '1Y': 365
  }

  const days = periods[selectedPeriod.value] || 30

  for (let i = 0; i < days; i++) {
    const date = new Date()
    date.setDate(date.getDate() - (days - i - 1))

    // Generate volatility with some clustering
    const baseVolatility = 25 + Math.sin(i / 10) * 10
    const noise = (Math.random() - 0.5) * 15
    const volatility = Math.max(5, Math.min(60, baseVolatility + noise))

    data.push({
      date: date.toISOString().split('T')[0],
      volatility,
      assets: {
        ETH: {
          volatility: volatility * (0.8 + Math.random() * 0.4),
          allocation: 35,
          name: 'Ethereum'
        },
        BTC: {
          volatility: volatility * (0.6 + Math.random() * 0.3),
          allocation: 30,
          name: 'Bitcoin'
        },
        UNI: {
          volatility: volatility * (1.2 + Math.random() * 0.6),
          allocation: 15,
          name: 'Uniswap'
        },
        AAVE: {
          volatility: volatility * (1.1 + Math.random() * 0.5),
          allocation: 12,
          name: 'Aave'
        },
        COMP: {
          volatility: volatility * (1.3 + Math.random() * 0.7),
          allocation: 8,
          name: 'Compound'
        }
      }
    })
  }

  return data
}

const volatilityData = computed(() => props.data.length ? props.data : generateMockVolatilityData())

const currentVolatility = computed(() => {
  const latest = volatilityData.value[volatilityData.value.length - 1]
  return latest ? latest.volatility : 0
})

const avgVolatility = computed(() => {
  if (volatilityData.value.length === 0) return 0
  const sum = volatilityData.value.reduce((total, d) => total + d.volatility, 0)
  return sum / volatilityData.value.length
})

const volatilityChange = computed(() => {
  if (volatilityData.value.length < 2) return 0
  const current = currentVolatility.value
  const previous = volatilityData.value[volatilityData.value.length - 2].volatility
  return ((current - previous) / previous) * 100
})

const maxVolatility = computed(() => {
  return Math.max(...volatilityData.value.map(d => d.volatility))
})

const minVolatility = computed(() => {
  return Math.min(...volatilityData.value.map(d => d.volatility))
})

const volatilityPoints = computed(() => {
  return volatilityData.value.map((d, index) => ({
    x: 40 + (index / (volatilityData.value.length - 1)) * 720,
    y: 220 - (d.volatility / (maxVolatility.value * 1.1)) * 200,
    value: d.volatility,
    date: d.date
  }))
})

const volatilityLinePath = computed(() => {
  return volatilityPoints.value.map((point, index) =>
    `${index === 0 ? 'M' : 'L'} ${point.x} ${point.y}`
  ).join(' ')
})

const volatilityAreaPath = computed(() => {
  const points = volatilityPoints.value
  if (points.length === 0) return ''

  const path = [`M 40 220`]
  points.forEach(point => path.push(`L ${point.x} ${point.y}`))
  path.push(`L ${points[points.length - 1].x} 220`)
  path.push('Z')

  return path.join(' ')
})

const thresholdY = computed(() => ({
  high: 220 - (highThreshold / (maxVolatility.value * 1.1)) * 200,
  low: 220 - (lowThreshold / (maxVolatility.value * 1.1)) * 200
}))

const assetVolatility = computed(() => {
  const latest = volatilityData.value[volatilityData.value.length - 1]
  if (!latest) return []

  return Object.entries(latest.assets).map(([symbol, data]) => ({
    symbol,
    ...data
  })).sort((a, b) => b.volatility - a.volatility)
})

const volatilityRegimes = computed(() => {
  const total = volatilityData.value.length
  if (total === 0) return []

  const low = volatilityData.value.filter(d => d.volatility <= 20).length
  const moderate = volatilityData.value.filter(d => d.volatility > 20 && d.volatility <= 35).length
  const high = volatilityData.value.filter(d => d.volatility > 35 && d.volatility <= 50).length
  const extreme = volatilityData.value.filter(d => d.volatility > 50).length

  return [
    {
      label: 'Low (≤20%)',
      percentage: Math.round((low / total) * 100),
      colorClass: 'bg-green-400'
    },
    {
      label: 'Moderate (20-35%)',
      percentage: Math.round((moderate / total) * 100),
      colorClass: 'bg-yellow-400'
    },
    {
      label: 'High (35-50%)',
      percentage: Math.round((high / total) * 100),
      colorClass: 'bg-orange-400'
    },
    {
      label: 'Extreme (>50%)',
      percentage: Math.round((extreme / total) * 100),
      colorClass: 'bg-red-400'
    }
  ]
})

const riskInsights = computed(() => {
  const insights = []
  const current = currentVolatility.value
  const avg = avgVolatility.value

  if (current > highThreshold) {
    insights.push(`Current volatility (${current.toFixed(1)}%) is in high-risk territory`)
  } else if (current < lowThreshold) {
    insights.push(`Current volatility (${current.toFixed(1)}%) indicates low market stress`)
  }

  if (current > avg * 1.5) {
    insights.push(`Volatility is ${((current / avg - 1) * 100).toFixed(0)}% above historical average`)
  }

  const highVolDays = volatilityData.value.filter(d => d.volatility > 35).length
  const totalDays = volatilityData.value.length
  if (highVolDays / totalDays > 0.3) {
    insights.push(`${Math.round((highVolDays / totalDays) * 100)}% of days show high volatility - consider risk management`)
  }

  const mostVolatileAsset = assetVolatility.value[0]
  if (mostVolatileAsset && mostVolatileAsset.volatility > current * 1.5) {
    insights.push(`${mostVolatileAsset.symbol} shows significantly higher volatility than portfolio average`)
  }

  return insights
})

const volatilityAlerts = computed(() => {
  const alerts: VolatilityAlert[] = []
  const current = currentVolatility.value

  if (current > 45) {
    alerts.push({
      id: '1',
      title: 'Extreme Volatility Alert',
      message: 'Portfolio volatility exceeds 45% - consider reducing risk exposure',
      severity: 'high'
    })
  } else if (current > 35) {
    alerts.push({
      id: '2',
      title: 'High Volatility Warning',
      message: 'Elevated volatility detected - monitor positions closely',
      severity: 'medium'
    })
  }

  if (volatilityChange.value > 50) {
    alerts.push({
      id: '3',
      title: 'Volatility Spike',
      message: `Volatility increased by ${volatilityChange.value.toFixed(0)}% from yesterday`,
      severity: 'medium'
    })
  }

  const volatileAssets = assetVolatility.value.filter(a => a.volatility > 50)
  if (volatileAssets.length > 0) {
    alerts.push({
      id: '4',
      title: 'High-Risk Assets Detected',
      message: `${volatileAssets.length} asset(s) showing extreme volatility (>50%)`,
      severity: 'medium'
    })
  }

  return alerts
})

// Methods
function getVolatilityColor(volatility: number): string {
  if (volatility <= 15) return '#10b981'
  if (volatility <= 25) return '#f59e0b'
  if (volatility <= 40) return '#f97316'
  return '#ef4444'
}

function getVolatilityLabel(volatility: number): string {
  if (volatility <= 15) return 'Low risk'
  if (volatility <= 25) return 'Moderate risk'
  if (volatility <= 40) return 'High risk'
  return 'Extreme risk'
}

function getVolatilityBarClass(volatility: number): string {
  if (volatility <= 15) return 'bg-green-400'
  if (volatility <= 25) return 'bg-yellow-400'
  if (volatility <= 40) return 'bg-orange-400'
  return 'bg-red-400'
}

function getVolatilityGradient(): string {
  return 'url(#volatilityGradient)'
}

function getAssetIcon(symbol: string): string {
  const iconMap: Record<string, string> = {
    ETH: '/tokens/eth.svg',
    BTC: '/tokens/btc.svg',
    UNI: '/tokens/uni.svg',
    AAVE: '/tokens/aave.svg',
    COMP: '/tokens/comp.svg'
  }
  return iconMap[symbol] || '/tokens/default.svg'
}

function showVolatilityTooltip(point: any, index: number) {
  tooltip.value = {
    show: true,
    x: point.x,
    y: point.y - 10,
    date: point.date,
    value: point.value
  }
}

function hideTooltip() {
  tooltip.value.show = false
}

function getChangeColor(change: number): string {
  return change >= 0 ? 'text-red-400' : 'text-green-400'
}

function getAlertClass(severity: string): string {
  switch (severity) {
    case 'high':
      return 'bg-red-500/10 border-red-500/30 text-red-400'
    case 'medium':
      return 'bg-yellow-500/10 border-yellow-500/30 text-yellow-400'
    default:
      return 'bg-blue-500/10 border-blue-500/30 text-blue-400'
  }
}

function getAlertIcon(severity: string): string {
  switch (severity) {
    case 'high':
      return 'ExclamationTriangleIcon'
    case 'medium':
      return 'ExclamationCircleIcon'
    default:
      return 'InformationCircleIcon'
  }
}
</script>

<style scoped>
/* Chart animations */
path {
  transition: all 0.3s ease;
}

circle {
  transition: all 0.2s ease;
}

/* Gradient definition */
svg defs {
  background: linear-gradient(
    180deg,
    rgba(239, 68, 68, 0.3) 0%,
    rgba(239, 68, 68, 0.1) 50%,
    rgba(239, 68, 68, 0.05) 100%
  );
}

/* Tooltip animations */
.tooltip-enter-active,
.tooltip-leave-active {
  transition: all 0.2s ease;
}

.tooltip-enter-from,
.tooltip-leave-to {
  opacity: 0;
  transform: translateY(-5px);
}
</style>