<template>
  <Card variant="glass">
    <template #header>
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center space-x-3">
          <HeroIcon name="ChartBarIcon" class="w-5 h-5 text-purple-400" />
          <div>
            <h3 class="text-lg font-semibold text-white">Market Comparison</h3>
            <p class="text-xs text-white/60">Portfolio vs benchmarks</p>
          </div>
        </div>

        <div class="flex items-center space-x-2">
          <select
            v-model="selectedPeriod"
            class="bg-slate-800/50 border border-slate-600/50 rounded-lg px-3 py-1 text-white text-xs focus:outline-none focus:border-moby-500/50"
          >
            <option value="1D">1D</option>
            <option value="7D">7D</option>
            <option value="30D">30D</option>
            <option value="90D">90D</option>
            <option value="1Y">1Y</option>
          </select>
        </div>
      </div>
    </template>

    <div class="space-y-6">
      <!-- Loading State -->
      <div v-if="loading" class="space-y-4">
        <div class="animate-pulse">
          <div class="h-64 bg-slate-700/30 rounded-lg"></div>
        </div>
        <div class="grid grid-cols-4 gap-4">
          <div v-for="i in 4" :key="i" class="animate-pulse">
            <div class="h-16 bg-slate-700/30 rounded-lg"></div>
          </div>
        </div>
      </div>

      <!-- Chart Container -->
      <div v-else class="relative">
        <div class="h-64 bg-slate-800/20 rounded-lg p-4">
          <svg class="w-full h-full" viewBox="0 0 800 240">
            <!-- Grid Lines -->
            <defs>
              <pattern id="grid" width="40" height="24" patternUnits="userSpaceOnUse">
                <path d="M 40 0 L 0 0 0 24" fill="none" stroke="rgba(255,255,255,0.1)" stroke-width="0.5"/>
              </pattern>
            </defs>
            <rect width="100%" height="100%" fill="url(#grid)" />

            <!-- Portfolio Line -->
            <path
              :d="portfolioPath"
              fill="none"
              stroke="#60a5fa"
              stroke-width="2"
              class="drop-shadow-sm"
            />

            <!-- S&P 500 Line -->
            <path
              :d="sp500Path"
              fill="none"
              stroke="#10b981"
              stroke-width="2"
              stroke-dasharray="5,5"
              class="opacity-80"
            />

            <!-- Bitcoin Line -->
            <path
              :d="bitcoinPath"
              fill="none"
              stroke="#f59e0b"
              stroke-width="2"
              stroke-dasharray="3,3"
              class="opacity-80"
            />

            <!-- Ethereum Line -->
            <path
              :d="ethereumPath"
              fill="none"
              stroke="#8b5cf6"
              stroke-width="2"
              stroke-dasharray="2,2"
              class="opacity-80"
            />

            <!-- Data Points -->
            <g v-for="(point, index) in portfolioData" :key="`portfolio-${index}`">
              <circle
                :cx="point.x"
                :cy="point.y"
                r="3"
                fill="#60a5fa"
                class="opacity-0 hover:opacity-100 transition-opacity cursor-pointer"
                @mouseenter="showTooltip(point, index, 'Portfolio')"
                @mouseleave="hideTooltip"
              />
            </g>
          </svg>

          <!-- Tooltip -->
          <div
            v-if="tooltip.show"
            :style="{ left: tooltip.x + 'px', top: tooltip.y + 'px' }"
            class="absolute z-10 bg-slate-800/90 backdrop-blur border border-white/20 rounded-lg p-3 text-xs pointer-events-none"
          >
            <div class="font-semibold text-white">{{ tooltip.label }}</div>
            <div class="text-white/70">{{ tooltip.date }}</div>
            <div :class="['font-bold', getReturnColor(tooltip.value)]">
              {{ tooltip.value >= 0 ? '+' : '' }}{{ tooltip.value.toFixed(2) }}%
            </div>
          </div>
        </div>

        <!-- Legend -->
        <div class="flex flex-wrap items-center justify-center gap-4 mt-4">
          <div class="flex items-center space-x-2">
            <div class="w-3 h-0.5 bg-blue-400"></div>
            <span class="text-xs text-white/70">Portfolio</span>
          </div>
          <div class="flex items-center space-x-2">
            <div class="w-3 h-0.5 bg-green-400 border-dashed border-t border-green-400"></div>
            <span class="text-xs text-white/70">S&P 500</span>
          </div>
          <div class="flex items-center space-x-2">
            <div class="w-3 h-0.5 bg-yellow-400 border-dashed border-t border-yellow-400"></div>
            <span class="text-xs text-white/70">Bitcoin</span>
          </div>
          <div class="flex items-center space-x-2">
            <div class="w-3 h-0.5 bg-purple-400 border-dashed border-t border-purple-400"></div>
            <span class="text-xs text-white/70">Ethereum</span>
          </div>
        </div>
      </div>

      <!-- Performance Comparison Table -->
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <div
          v-for="benchmark in benchmarks"
          :key="benchmark.name"
          class="bg-slate-800/30 rounded-lg p-4 text-center"
        >
          <div class="flex items-center justify-center space-x-2 mb-2">
            <div :class="benchmark.colorClass" class="w-3 h-3 rounded-full"></div>
            <span class="text-sm font-medium text-white">{{ benchmark.name }}</span>
          </div>
          <div :class="['text-xl font-bold', getReturnColor(benchmark.return)]">
            {{ benchmark.return >= 0 ? '+' : '' }}{{ benchmark.return.toFixed(1) }}%
          </div>
          <div class="text-xs text-white/60 mt-1">{{ selectedPeriod }} Return</div>

          <!-- Relative Performance -->
          <div class="mt-2 pt-2 border-t border-white/10">
            <div class="text-xs text-white/60">vs Portfolio</div>
            <div :class="['text-sm font-medium', getRelativePerformanceColor(benchmark.relative)]">
              {{ benchmark.relative >= 0 ? '+' : '' }}{{ benchmark.relative.toFixed(1) }}%
            </div>
          </div>
        </div>
      </div>

      <!-- Key Metrics -->
      <div class="grid grid-cols-2 lg:grid-cols-3 gap-4">
        <div class="bg-slate-800/30 rounded-lg p-4">
          <div class="flex items-center space-x-2 mb-2">
            <HeroIcon name="TrophyIcon" class="w-4 h-4 text-yellow-400" />
            <span class="text-sm text-white/60">Best Performer</span>
          </div>
          <div class="text-lg font-bold text-green-400">{{ bestPerformer.name }}</div>
          <div class="text-xs text-white/60">{{ bestPerformer.return.toFixed(1) }}% return</div>
        </div>

        <div class="bg-slate-800/30 rounded-lg p-4">
          <div class="flex items-center space-x-2 mb-2">
            <HeroIcon name="ChartBarIcon" class="w-4 h-4 text-blue-400" />
            <span class="text-sm text-white/60">Correlation to S&P</span>
          </div>
          <div :class="['text-lg font-bold', getCorrelationColor(spCorrelation)]">
            {{ spCorrelation.toFixed(2) }}
          </div>
          <div class="text-xs text-white/60">{{ getCorrelationLabel(spCorrelation) }}</div>
        </div>

        <div class="bg-slate-800/30 rounded-lg p-4">
          <div class="flex items-center space-x-2 mb-2">
            <HeroIcon name="ScaleIcon" class="w-4 h-4 text-purple-400" />
            <span class="text-sm text-white/60">Risk-Adjusted Return</span>
          </div>
          <div :class="['text-lg font-bold', getSharpeColor(riskAdjustedReturn)]">
            {{ riskAdjustedReturn.toFixed(2) }}
          </div>
          <div class="text-xs text-white/60">Sharpe ratio</div>
        </div>
      </div>

      <!-- Market Insights -->
      <div class="bg-slate-800/20 rounded-lg p-4">
        <h4 class="text-sm font-semibold text-white mb-3 flex items-center space-x-2">
          <HeroIcon name="LightBulbIcon" class="w-4 h-4 text-yellow-400" />
          <span>Market Insights</span>
        </h4>
        <div class="space-y-2 text-sm text-white/70">
          <p v-for="insight in marketInsights" :key="insight" class="flex items-start space-x-2">
            <HeroIcon name="ChevronRightIcon" class="w-3 h-3 mt-0.5 text-moby-400 flex-shrink-0" />
            <span>{{ insight }}</span>
          </p>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'

import Card from '@components/ui/Card.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'

interface ChartData {
  date: string
  portfolio: number
  sp500: number
  bitcoin: number
  ethereum: number
}

interface Props {
  data: ChartData[]
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

const selectedPeriod = ref('30D')
const tooltip = ref({
  show: false,
  x: 0,
  y: 0,
  label: '',
  date: '',
  value: 0
})

// Generate mock data for demonstration
const generateMockData = () => {
  const data: ChartData[] = []
  const periods: Record<string, number> = {
    '1D': 1,
    '7D': 7,
    '30D': 30,
    '90D': 90,
    '1Y': 365
  }

  const days = periods[selectedPeriod.value] || 30
  let portfolioValue = 0
  let sp500Value = 0
  let bitcoinValue = 0
  let ethereumValue = 0

  for (let i = 0; i < days; i++) {
    const date = new Date()
    date.setDate(date.getDate() - (days - i - 1))

    portfolioValue += (Math.random() - 0.45) * 2
    sp500Value += (Math.random() - 0.48) * 1.5
    bitcoinValue += (Math.random() - 0.4) * 4
    ethereumValue += (Math.random() - 0.42) * 3.5

    data.push({
      date: date.toISOString().split('T')[0],
      portfolio: portfolioValue,
      sp500: sp500Value,
      bitcoin: bitcoinValue,
      ethereum: ethereumValue
    })
  }

  return data
}

const chartData = computed(() => props.data.length ? props.data : generateMockData())

// SVG path generation
const portfolioData = computed(() => {
  return chartData.value.map((point, index) => ({
    x: (index / (chartData.value.length - 1)) * 760 + 20,
    y: 200 - ((point.portfolio + 20) / 40) * 160,
    value: point.portfolio,
    date: point.date
  }))
})

const portfolioPath = computed(() => {
  return portfolioData.value.map((point, index) =>
    `${index === 0 ? 'M' : 'L'} ${point.x} ${point.y}`
  ).join(' ')
})

const sp500Path = computed(() => {
  return chartData.value.map((point, index) => {
    const x = (index / (chartData.value.length - 1)) * 760 + 20
    const y = 200 - ((point.sp500 + 20) / 40) * 160
    return `${index === 0 ? 'M' : 'L'} ${x} ${y}`
  }).join(' ')
})

const bitcoinPath = computed(() => {
  return chartData.value.map((point, index) => {
    const x = (index / (chartData.value.length - 1)) * 760 + 20
    const y = 200 - ((point.bitcoin + 20) / 40) * 160
    return `${index === 0 ? 'M' : 'L'} ${x} ${y}`
  }).join(' ')
})

const ethereumPath = computed(() => {
  return chartData.value.map((point, index) => {
    const x = (index / (chartData.value.length - 1)) * 760 + 20
    const y = 200 - ((point.ethereum + 20) / 40) * 160
    return `${index === 0 ? 'M' : 'L'} ${x} ${y}`
  }).join(' ')
})

// Benchmarks comparison
const benchmarks = computed(() => {
  const latest = chartData.value[chartData.value.length - 1]
  if (!latest) return []

  return [
    {
      name: 'Portfolio',
      return: latest.portfolio,
      relative: 0,
      colorClass: 'bg-blue-400'
    },
    {
      name: 'S&P 500',
      return: latest.sp500,
      relative: latest.sp500 - latest.portfolio,
      colorClass: 'bg-green-400'
    },
    {
      name: 'Bitcoin',
      return: latest.bitcoin,
      relative: latest.bitcoin - latest.portfolio,
      colorClass: 'bg-yellow-400'
    },
    {
      name: 'Ethereum',
      return: latest.ethereum,
      relative: latest.ethereum - latest.portfolio,
      colorClass: 'bg-purple-400'
    }
  ]
})

const bestPerformer = computed(() => {
  return benchmarks.value.reduce((best, current) =>
    current.return > best.return ? current : best
  )
})

const spCorrelation = computed(() => {
  // Simple correlation calculation
  if (chartData.value.length < 2) return 0

  const portfolioReturns = chartData.value.map(d => d.portfolio)
  const sp500Returns = chartData.value.map(d => d.sp500)

  const avgPortfolio = portfolioReturns.reduce((a, b) => a + b) / portfolioReturns.length
  const avgSp500 = sp500Returns.reduce((a, b) => a + b) / sp500Returns.length

  let numerator = 0
  let portfolioSumSq = 0
  let sp500SumSq = 0

  for (let i = 0; i < portfolioReturns.length; i++) {
    const portfolioDiff = portfolioReturns[i] - avgPortfolio
    const sp500Diff = sp500Returns[i] - avgSp500

    numerator += portfolioDiff * sp500Diff
    portfolioSumSq += portfolioDiff * portfolioDiff
    sp500SumSq += sp500Diff * sp500Diff
  }

  const denominator = Math.sqrt(portfolioSumSq * sp500SumSq)
  return denominator === 0 ? 0 : numerator / denominator
})

const riskAdjustedReturn = computed(() => {
  if (chartData.value.length < 2) return 0

  const returns = chartData.value.map(d => d.portfolio)
  const avgReturn = returns.reduce((a, b) => a + b) / returns.length
  const variance = returns.reduce((sum, ret) => sum + Math.pow(ret - avgReturn, 2), 0) / returns.length
  const volatility = Math.sqrt(variance)

  return volatility === 0 ? 0 : avgReturn / volatility
})

const marketInsights = computed(() => {
  const insights = []
  const latest = chartData.value[chartData.value.length - 1]

  if (!latest) return insights

  if (latest.portfolio > latest.sp500) {
    insights.push(`Portfolio outperforming S&P 500 by ${(latest.portfolio - latest.sp500).toFixed(1)}%`)
  } else {
    insights.push(`Portfolio underperforming S&P 500 by ${(latest.sp500 - latest.portfolio).toFixed(1)}%`)
  }

  if (spCorrelation.value > 0.7) {
    insights.push('High correlation with traditional markets suggests reduced diversification')
  } else if (spCorrelation.value < 0.3) {
    insights.push('Low correlation with traditional markets indicates good diversification')
  }

  if (riskAdjustedReturn.value > 1.5) {
    insights.push('Strong risk-adjusted returns indicate efficient portfolio management')
  }

  const cryptoOutperformance = (latest.bitcoin + latest.ethereum) / 2 - latest.portfolio
  if (cryptoOutperformance > 5) {
    insights.push('Crypto assets significantly outperforming - consider rebalancing')
  }

  return insights
})

// Methods
function showTooltip(point: any, index: number, label: string) {
  tooltip.value = {
    show: true,
    x: point.x,
    y: point.y - 10,
    label,
    date: point.date,
    value: point.value
  }
}

function hideTooltip() {
  tooltip.value.show = false
}

function getReturnColor(value: number): string {
  return value >= 0 ? 'text-green-400' : 'text-red-400'
}

function getRelativePerformanceColor(value: number): string {
  if (value > 2) return 'text-green-400'
  if (value < -2) return 'text-red-400'
  return 'text-yellow-400'
}

function getCorrelationColor(value: number): string {
  if (value > 0.7) return 'text-red-400'
  if (value < 0.3) return 'text-green-400'
  return 'text-yellow-400'
}

function getCorrelationLabel(value: number): string {
  if (value > 0.7) return 'High correlation'
  if (value < 0.3) return 'Low correlation'
  return 'Moderate correlation'
}

function getSharpeColor(value: number): string {
  if (value > 1.5) return 'text-green-400'
  if (value > 1) return 'text-yellow-400'
  return 'text-red-400'
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