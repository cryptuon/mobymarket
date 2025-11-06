<template>
  <Card variant="glass">
    <template #header>
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center space-x-3">
          <HeroIcon name="ChartBarIcon" class="w-5 h-5 text-blue-400" />
          <div>
            <h3 class="text-lg font-semibold text-white">Portfolio Performance</h3>
            <p class="text-xs text-white/60">Value over time</p>
          </div>
        </div>

        <div class="flex items-center space-x-2">
          <!-- Chart Type -->
          <div class="flex bg-slate-800/50 rounded-lg p-1">
            <button
              v-for="type in chartTypes"
              :key="type.value"
              @click="chartType = type.value"
              :class="[
                'px-3 py-1 rounded-md text-xs font-medium transition-all duration-200',
                chartType === type.value
                  ? 'bg-moby-500 text-white'
                  : 'text-white/60 hover:text-white hover:bg-white/10'
              ]"
            >
              {{ type.label }}
            </button>
          </div>

          <!-- Benchmark Toggle -->
          <button
            @click="showBenchmark = !showBenchmark"
            :class="[
              'px-3 py-1 rounded-lg text-xs font-medium transition-all duration-200',
              showBenchmark
                ? 'bg-purple-500/20 text-purple-400 border border-purple-500/30'
                : 'text-white/60 hover:text-white hover:bg-white/10 border border-transparent'
            ]"
          >
            Benchmark
          </button>
        </div>
      </div>
    </template>

    <div class="space-y-4">
      <!-- Performance Stats -->
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <div class="bg-slate-800/30 rounded-lg p-3">
          <div class="text-xs text-white/60 mb-1">Total Return</div>
          <div :class="['text-lg font-bold', getReturnColor(performanceStats.totalReturn)]">
            {{ performanceStats.totalReturn >= 0 ? '+' : '' }}{{ performanceStats.totalReturn.toFixed(2) }}%
          </div>
        </div>

        <div class="bg-slate-800/30 rounded-lg p-3">
          <div class="text-xs text-white/60 mb-1">Current Value</div>
          <div class="text-lg font-bold text-white">${{ formatCurrency(performanceStats.currentValue) }}</div>
        </div>

        <div class="bg-slate-800/30 rounded-lg p-3">
          <div class="text-xs text-white/60 mb-1">Best Day</div>
          <div class="text-lg font-bold text-green-400">+{{ performanceStats.bestDay.toFixed(2) }}%</div>
        </div>

        <div class="bg-slate-800/30 rounded-lg p-3">
          <div class="text-xs text-white/60 mb-1">Worst Day</div>
          <div class="text-lg font-bold text-red-400">{{ performanceStats.worstDay.toFixed(2) }}%</div>
        </div>
      </div>

      <!-- Chart Container -->
      <div class="relative h-80 bg-slate-800/20 rounded-xl p-4">
        <!-- Loading State -->
        <div v-if="loading" class="absolute inset-0 flex items-center justify-center bg-slate-800/50 backdrop-blur-sm rounded-xl">
          <div class="text-center">
            <div class="animate-spin rounded-full h-8 w-8 border-2 border-white/20 border-t-white mx-auto mb-2"></div>
            <p class="text-white/60 text-sm">Loading performance data...</p>
          </div>
        </div>

        <!-- Chart -->
        <div v-else class="h-full relative">
          <!-- Y-Axis Labels -->
          <div class="absolute left-0 top-0 bottom-8 w-20 flex flex-col justify-between text-xs text-white/60">
            <span>${{ formatCurrency(maxValue) }}</span>
            <span>${{ formatCurrency(maxValue * 0.75) }}</span>
            <span>${{ formatCurrency(maxValue * 0.5) }}</span>
            <span>${{ formatCurrency(maxValue * 0.25) }}</span>
            <span>${{ formatCurrency(minValue) }}</span>
          </div>

          <!-- Chart Area -->
          <div class="ml-20 h-full relative">
            <!-- Grid Lines -->
            <div class="absolute inset-0 grid grid-rows-4 opacity-20">
              <div v-for="i in 4" :key="i" class="border-b border-white/20"></div>
            </div>

            <!-- Performance Line -->
            <svg
              class="absolute inset-0 w-full h-full"
              :viewBox="`0 0 ${data.length * 10} 100`"
              preserveAspectRatio="none"
            >
              <!-- Portfolio Line -->
              <path
                :d="portfolioPath"
                stroke="url(#portfolioGradient)"
                stroke-width="2"
                fill="none"
                class="transition-all duration-500"
              />

              <!-- Benchmark Line -->
              <path
                v-if="showBenchmark && benchmark"
                :d="benchmarkPath"
                stroke="url(#benchmarkGradient)"
                stroke-width="1.5"
                stroke-dasharray="5,5"
                fill="none"
                class="transition-all duration-500"
              />

              <!-- Area Fill -->
              <path
                v-if="chartType === 'area'"
                :d="areaPath"
                fill="url(#areaGradient)"
                class="transition-all duration-500"
              />

              <!-- Data Points -->
              <circle
                v-for="(point, index) in visibleDataPoints"
                :key="index"
                :cx="(index / (data.length - 1)) * (data.length * 10)"
                :cy="100 - getYPosition(point.portfolioValue)"
                r="3"
                :fill="getPointColor(index)"
                class="opacity-0 hover:opacity-100 transition-opacity cursor-pointer"
                @mouseover="showTooltip(point, index, $event)"
                @mouseleave="hideTooltip"
              />

              <!-- Gradients -->
              <defs>
                <linearGradient id="portfolioGradient" x1="0%" y1="0%" x2="100%" y2="0%">
                  <stop offset="0%" style="stop-color:#3B82F6;stop-opacity:1" />
                  <stop offset="50%" style="stop-color:#8B5CF6;stop-opacity:1" />
                  <stop offset="100%" style="stop-color:#06B6D4;stop-opacity:1" />
                </linearGradient>

                <linearGradient id="benchmarkGradient" x1="0%" y1="0%" x2="100%" y2="0%">
                  <stop offset="0%" style="stop-color:#64748B;stop-opacity:0.8" />
                  <stop offset="100%" style="stop-color:#94A3B8;stop-opacity:0.8" />
                </linearGradient>

                <linearGradient id="areaGradient" x1="0%" y1="0%" x2="0%" y2="100%">
                  <stop offset="0%" style="stop-color:#3B82F6;stop-opacity:0.3" />
                  <stop offset="100%" style="stop-color:#3B82F6;stop-opacity:0.05" />
                </linearGradient>
              </defs>
            </svg>

            <!-- X-Axis Labels -->
            <div class="absolute bottom-0 left-0 right-0 h-8 flex items-center justify-between text-xs text-white/60">
              <span v-for="(label, index) in xAxisLabels" :key="index">
                {{ label }}
              </span>
            </div>
          </div>

          <!-- Tooltip -->
          <Transition
            name="tooltip"
            enter-active-class="transition-all duration-150"
            enter-from-class="opacity-0 scale-95"
            enter-to-class="opacity-100 scale-100"
            leave-active-class="transition-all duration-100"
            leave-from-class="opacity-100 scale-100"
            leave-to-class="opacity-0 scale-95"
          >
            <div
              v-if="tooltip.show"
              :style="{ left: `${tooltip.x}px`, top: `${tooltip.y}px` }"
              class="absolute z-10 bg-slate-800/90 backdrop-blur border border-white/20 rounded-lg p-3 min-w-40 pointer-events-none"
            >
              <div class="text-xs text-white/60 mb-1">{{ tooltip.date }}</div>
              <div class="text-sm font-semibold text-white">${{ formatCurrency(tooltip.value) }}</div>
              <div :class="['text-xs', getReturnColor(tooltip.return)]">
                {{ tooltip.return >= 0 ? '+' : '' }}{{ tooltip.return.toFixed(2) }}%
              </div>
              <div v-if="tooltip.benchmark" class="text-xs text-white/60 mt-1">
                Benchmark: ${{ formatCurrency(tooltip.benchmark) }}
              </div>
            </div>
          </Transition>
        </div>
      </div>

      <!-- Legend -->
      <div class="flex items-center justify-between pt-4 border-t border-white/10">
        <div class="flex items-center space-x-4 text-xs text-white/60">
          <div class="flex items-center space-x-2">
            <div class="w-3 h-0.5 bg-gradient-to-r from-blue-500 to-purple-500"></div>
            <span>Portfolio</span>
          </div>
          <div v-if="showBenchmark" class="flex items-center space-x-2">
            <div class="w-3 h-0.5 bg-slate-400 opacity-80" style="border-top: 1px dashed;"></div>
            <span>Benchmark</span>
          </div>
        </div>

        <div class="text-xs text-white/60">
          {{ data.length }} data points
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

import Card from '@components/ui/Card.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'

interface PerformanceData {
  timestamp: string
  portfolioValue: number
  benchmark?: number
  returns?: number
}

interface Props {
  data: PerformanceData[]
  timeRange: string
  benchmark?: boolean
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  benchmark: false,
  loading: false
})

const chartType = ref<'line' | 'area'>('line')
const showBenchmark = ref(props.benchmark)

const chartTypes = [
  { value: 'line', label: 'Line' },
  { value: 'area', label: 'Area' }
]

const tooltip = ref({
  show: false,
  x: 0,
  y: 0,
  value: 0,
  date: '',
  return: 0,
  benchmark: 0
})

// Computed properties
const performanceStats = computed(() => {
  if (props.data.length === 0) return {
    totalReturn: 0,
    currentValue: 0,
    bestDay: 0,
    worstDay: 0
  }

  const firstValue = props.data[0].portfolioValue
  const lastValue = props.data[props.data.length - 1].portfolioValue
  const totalReturn = ((lastValue - firstValue) / firstValue) * 100

  const dailyReturns = props.data.slice(1).map((point, index) => {
    const prevValue = props.data[index].portfolioValue
    return ((point.portfolioValue - prevValue) / prevValue) * 100
  })

  const bestDay = Math.max(...dailyReturns)
  const worstDay = Math.min(...dailyReturns)

  return {
    totalReturn,
    currentValue: lastValue,
    bestDay,
    worstDay
  }
})

const minValue = computed(() => {
  if (props.data.length === 0) return 0
  const values = props.data.map(d => d.portfolioValue)
  if (showBenchmark.value && props.data[0].benchmark) {
    values.push(...props.data.map(d => d.benchmark || 0))
  }
  return Math.min(...values) * 0.95 // Add 5% padding
})

const maxValue = computed(() => {
  if (props.data.length === 0) return 1000000
  const values = props.data.map(d => d.portfolioValue)
  if (showBenchmark.value && props.data[0].benchmark) {
    values.push(...props.data.map(d => d.benchmark || 0))
  }
  return Math.max(...values) * 1.05 // Add 5% padding
})

const portfolioPath = computed(() => {
  if (props.data.length === 0) return ''

  const points = props.data.map((point, index) => {
    const x = (index / (props.data.length - 1)) * (props.data.length * 10)
    const y = 100 - getYPosition(point.portfolioValue)
    return `${x},${y}`
  })

  return `M ${points.join(' L ')}`
})

const benchmarkPath = computed(() => {
  if (props.data.length === 0 || !props.data[0].benchmark) return ''

  const points = props.data.map((point, index) => {
    const x = (index / (props.data.length - 1)) * (props.data.length * 10)
    const y = 100 - getYPosition(point.benchmark || 0)
    return `${x},${y}`
  })

  return `M ${points.join(' L ')}`
})

const areaPath = computed(() => {
  if (props.data.length === 0) return ''

  const line = portfolioPath.value
  const width = props.data.length * 10
  return `${line} L ${width},100 L 0,100 Z`
})

const xAxisLabels = computed(() => {
  if (props.data.length === 0) return []

  const step = Math.max(1, Math.floor(props.data.length / 6))
  return props.data
    .filter((_, index) => index % step === 0)
    .map(point => formatDateLabel(point.timestamp))
})

const visibleDataPoints = computed(() => {
  // Show every nth point to avoid overcrowding
  const step = Math.max(1, Math.floor(props.data.length / 50))
  return props.data.filter((_, index) => index % step === 0)
})

// Methods
function getYPosition(value: number): number {
  const range = maxValue.value - minValue.value
  if (range === 0) return 50
  return ((value - minValue.value) / range) * 100
}

function getPointColor(index: number): string {
  const colors = ['#3B82F6', '#8B5CF6', '#06B6D4']
  return colors[index % colors.length]
}

function getReturnColor(returnValue: number): string {
  return returnValue >= 0 ? 'text-green-400' : 'text-red-400'
}

function formatCurrency(amount: number): string {
  if (amount >= 1e9) return `${(amount / 1e9).toFixed(2)}B`
  if (amount >= 1e6) return `${(amount / 1e6).toFixed(2)}M`
  if (amount >= 1e3) return `${(amount / 1e3).toFixed(2)}K`
  return amount.toLocaleString()
}

function formatDateLabel(timestamp: string): string {
  const date = new Date(timestamp)

  if (props.timeRange === '24h') {
    return date.toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit'
    })
  } else if (props.timeRange === '7d') {
    return date.toLocaleDateString('en-US', {
      weekday: 'short'
    })
  } else {
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric'
    })
  }
}

function showTooltip(point: PerformanceData, index: number, event: MouseEvent) {
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect()
  const containerRect = (event.currentTarget as HTMLElement).closest('.relative')?.getBoundingClientRect()

  if (containerRect) {
    tooltip.value = {
      show: true,
      x: rect.left - containerRect.left,
      y: rect.top - containerRect.top - 10,
      value: point.portfolioValue,
      date: new Date(point.timestamp).toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
        year: 'numeric'
      }),
      return: point.returns || 0,
      benchmark: point.benchmark || 0
    }
  }
}

function hideTooltip() {
  tooltip.value.show = false
}
</script>

<style scoped>
/* Tooltip animations */
.tooltip-enter-active,
.tooltip-leave-active {
  transition: all 0.15s ease;
}

.tooltip-enter-from,
.tooltip-leave-to {
  opacity: 0;
  transform: scale(0.95);
}

/* Smooth transitions for chart elements */
path {
  transition: all 0.5s ease;
}

circle {
  transition: all 0.3s ease;
}

/* Custom dashed line for benchmark */
.dashed-line {
  stroke-dasharray: 5, 5;
}
</style>