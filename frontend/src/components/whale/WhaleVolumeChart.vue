<template>
  <Card variant="glass">
    <template #header>
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center space-x-3">
          <HeroIcon name="ChartBarIcon" class="w-5 h-5 text-blue-400" />
          <div>
            <h3 class="text-lg font-semibold text-white">Whale Volume Chart</h3>
            <p class="text-xs text-white/60">Trading volume over time</p>
          </div>
        </div>

        <div class="flex items-center space-x-2">
          <!-- Chart Type Toggle -->
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

          <!-- Metric Toggle -->
          <select
            v-model="selectedMetric"
            class="bg-slate-800/50 border border-slate-600/50 rounded-lg px-3 py-1 text-white text-xs focus:outline-none focus:border-moby-500/50"
          >
            <option value="volume">Volume</option>
            <option value="trades">Trade Count</option>
            <option value="whales">Active Whales</option>
            <option value="average">Avg Trade Size</option>
          </select>
        </div>
      </div>
    </template>

    <div class="space-y-4">
      <!-- Chart Stats -->
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <div class="bg-slate-800/30 rounded-lg p-3">
          <div class="text-xs text-white/60 mb-1">Peak Volume</div>
          <div class="text-lg font-bold text-white">${{ formatCurrency(peakVolume) }}</div>
          <div class="text-xs text-green-400">{{ peakVolumeTime }}</div>
        </div>

        <div class="bg-slate-800/30 rounded-lg p-3">
          <div class="text-xs text-white/60 mb-1">Average</div>
          <div class="text-lg font-bold text-white">${{ formatCurrency(averageVolume) }}</div>
          <div class="text-xs text-blue-400">{{ getMetricUnit() }}</div>
        </div>

        <div class="bg-slate-800/30 rounded-lg p-3">
          <div class="text-xs text-white/60 mb-1">Total {{ timeRange.toUpperCase() }}</div>
          <div class="text-lg font-bold text-white">${{ formatCurrency(totalVolume) }}</div>
          <div class="text-xs text-purple-400">{{ timeRange }} period</div>
        </div>

        <div class="bg-slate-800/30 rounded-lg p-3">
          <div class="text-xs text-white/60 mb-1">Trend</div>
          <div class="text-lg font-bold" :class="trendColorClass">
            {{ trend >= 0 ? '+' : '' }}{{ trend.toFixed(1) }}%
          </div>
          <div class="text-xs text-white/60">vs previous period</div>
        </div>
      </div>

      <!-- Chart Container -->
      <div class="relative h-80 bg-slate-800/20 rounded-xl p-4 overflow-hidden">
        <!-- Loading State -->
        <div v-if="isLoading" class="absolute inset-0 flex items-center justify-center bg-slate-800/50 backdrop-blur-sm rounded-xl">
          <div class="text-center">
            <div class="animate-spin rounded-full h-8 w-8 border-2 border-white/20 border-t-white mx-auto mb-2"></div>
            <p class="text-white/60 text-sm">Loading chart data...</p>
          </div>
        </div>

        <!-- Chart -->
        <div v-else class="h-full relative">
          <!-- Y-Axis Labels -->
          <div class="absolute left-0 top-0 bottom-8 w-16 flex flex-col justify-between text-xs text-white/60">
            <span>${{ formatCurrency(maxValue) }}</span>
            <span>${{ formatCurrency(maxValue * 0.75) }}</span>
            <span>${{ formatCurrency(maxValue * 0.5) }}</span>
            <span>${{ formatCurrency(maxValue * 0.25) }}</span>
            <span>$0</span>
          </div>

          <!-- Chart Area -->
          <div class="ml-16 h-full relative">
            <!-- Grid Lines -->
            <div class="absolute inset-0 grid grid-rows-4 opacity-20">
              <div v-for="i in 4" :key="i" class="border-b border-white/20"></div>
            </div>

            <!-- Chart Bars/Line -->
            <div class="absolute inset-0 flex items-end space-x-1 pb-8">
              <div
                v-for="(point, index) in chartData"
                :key="index"
                class="flex-1 flex flex-col items-center group cursor-pointer"
                @mouseover="showTooltip(point, index, $event)"
                @mouseleave="hideTooltip"
              >
                <!-- Bar Chart -->
                <div
                  v-if="chartType === 'bar'"
                  :class="getBarClass(point, index)"
                  :style="{ height: `${getBarHeight(point)}%` }"
                  class="w-full rounded-t-sm transition-all duration-200 hover:opacity-80"
                ></div>

                <!-- Line Chart Point -->
                <div
                  v-if="chartType === 'line'"
                  :class="getPointClass(point, index)"
                  :style="{ bottom: `${getBarHeight(point)}%` }"
                  class="absolute w-2 h-2 rounded-full transition-all duration-200"
                ></div>
              </div>

              <!-- Line Chart Connections -->
              <svg
                v-if="chartType === 'line'"
                class="absolute inset-0 pointer-events-none"
                :viewBox="`0 0 ${chartData.length * 20} 100`"
                preserveAspectRatio="none"
              >
                <path
                  :d="linePath"
                  stroke="url(#lineGradient)"
                  stroke-width="2"
                  fill="none"
                  class="transition-all duration-500"
                />
                <defs>
                  <linearGradient id="lineGradient" x1="0%" y1="0%" x2="100%" y2="0%">
                    <stop offset="0%" style="stop-color:#3B82F6;stop-opacity:1" />
                    <stop offset="50%" style="stop-color:#8B5CF6;stop-opacity:1" />
                    <stop offset="100%" style="stop-color:#06B6D4;stop-opacity:1" />
                  </linearGradient>
                </defs>
              </svg>

              <!-- Area Fill for Line Chart -->
              <svg
                v-if="chartType === 'area'"
                class="absolute inset-0 pointer-events-none"
                :viewBox="`0 0 ${chartData.length * 20} 100`"
                preserveAspectRatio="none"
              >
                <path
                  :d="areaPath"
                  fill="url(#areaGradient)"
                  class="transition-all duration-500"
                />
                <defs>
                  <linearGradient id="areaGradient" x1="0%" y1="0%" x2="0%" y2="100%">
                    <stop offset="0%" style="stop-color:#3B82F6;stop-opacity:0.3" />
                    <stop offset="100%" style="stop-color:#3B82F6;stop-opacity:0.05" />
                  </linearGradient>
                </defs>
              </svg>
            </div>

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
              class="absolute z-10 bg-slate-800/90 backdrop-blur border border-white/20 rounded-lg p-3 min-w-32 pointer-events-none"
            >
              <div class="text-xs text-white/60 mb-1">{{ tooltip.time }}</div>
              <div class="text-sm font-semibold text-white">${{ formatCurrency(tooltip.value) }}</div>
              <div v-if="tooltip.trades" class="text-xs text-white/60 mt-1">
                {{ tooltip.trades }} trades
              </div>
              <div v-if="tooltip.whales" class="text-xs text-white/60">
                {{ tooltip.whales }} whales
              </div>
            </div>
          </Transition>
        </div>
      </div>

      <!-- Chart Controls -->
      <div class="flex items-center justify-between pt-4 border-t border-white/10">
        <div class="flex items-center space-x-2 text-xs text-white/60">
          <div class="flex items-center space-x-1">
            <div class="w-3 h-3 bg-gradient-to-r from-blue-500 to-purple-500 rounded-sm"></div>
            <span>Whale Volume</span>
          </div>
        </div>

        <div class="flex items-center space-x-3">
          <button
            @click="zoomIn"
            class="p-1 hover:bg-white/10 rounded transition-colors"
            title="Zoom In"
          >
            <HeroIcon name="MagnifyingGlassPlusIcon" class="w-4 h-4 text-white/70" />
          </button>
          <button
            @click="zoomOut"
            class="p-1 hover:bg-white/10 rounded transition-colors"
            title="Zoom Out"
          >
            <HeroIcon name="MagnifyingGlassMinusIcon" class="w-4 h-4 text-white/70" />
          </button>
          <button
            @click="resetZoom"
            class="p-1 hover:bg-white/10 rounded transition-colors"
            title="Reset Zoom"
          >
            <HeroIcon name="ArrowsPointingOutIcon" class="w-4 h-4 text-white/70" />
          </button>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'

import Card from '@components/ui/Card.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'

interface ChartDataPoint {
  timestamp: string
  volume: number
  trades: number
  whales: number
}

interface Props {
  data: ChartDataPoint[]
  timeRange: string
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

const chartType = ref<'bar' | 'line' | 'area'>('bar')
const selectedMetric = ref('volume')
const isLoading = ref(false)
const zoomLevel = ref(1)

const chartTypes = [
  { value: 'bar', label: 'Bar' },
  { value: 'line', label: 'Line' },
  { value: 'area', label: 'Area' }
]

const tooltip = ref({
  show: false,
  x: 0,
  y: 0,
  value: 0,
  time: '',
  trades: 0,
  whales: 0
})

// Computed properties
const chartData = computed(() => {
  if (!props.data || props.data.length === 0) return []

  // Apply zoom by taking a subset of data
  const startIndex = Math.max(0, Math.floor((props.data.length - 1) * (1 - 1/zoomLevel.value)))
  const endIndex = props.data.length - 1

  return props.data.slice(startIndex, endIndex + 1)
})

const maxValue = computed(() => {
  if (chartData.value.length === 0) return 100

  return Math.max(...chartData.value.map(point => {
    switch (selectedMetric.value) {
      case 'trades': return point.trades
      case 'whales': return point.whales
      case 'average': return point.volume / Math.max(1, point.trades)
      default: return point.volume
    }
  }))
})

const peakVolume = computed(() => {
  if (chartData.value.length === 0) return 0
  return Math.max(...chartData.value.map(p => p.volume))
})

const peakVolumeTime = computed(() => {
  if (chartData.value.length === 0) return 'N/A'
  const peak = chartData.value.find(p => p.volume === peakVolume.value)
  return peak ? formatTimeLabel(peak.timestamp) : 'N/A'
})

const averageVolume = computed(() => {
  if (chartData.value.length === 0) return 0
  const sum = chartData.value.reduce((acc, point) => {
    switch (selectedMetric.value) {
      case 'trades': return acc + point.trades
      case 'whales': return acc + point.whales
      case 'average': return acc + (point.volume / Math.max(1, point.trades))
      default: return acc + point.volume
    }
  }, 0)
  return sum / chartData.value.length
})

const totalVolume = computed(() => {
  return chartData.value.reduce((sum, point) => sum + point.volume, 0)
})

const trend = computed(() => {
  if (chartData.value.length < 2) return 0
  const first = chartData.value[0].volume
  const last = chartData.value[chartData.value.length - 1].volume
  return first > 0 ? ((last - first) / first) * 100 : 0
})

const trendColorClass = computed(() => {
  return trend.value >= 0 ? 'text-green-400' : 'text-red-400'
})

const xAxisLabels = computed(() => {
  if (chartData.value.length === 0) return []

  const step = Math.max(1, Math.floor(chartData.value.length / 6))
  return chartData.value
    .filter((_, index) => index % step === 0)
    .map(point => formatTimeLabel(point.timestamp))
})

const linePath = computed(() => {
  if (chartData.value.length === 0) return ''

  const width = chartData.value.length * 20
  const points = chartData.value.map((point, index) => {
    const x = (index / (chartData.value.length - 1)) * width
    const y = 100 - getBarHeight(point)
    return `${x},${y}`
  })

  return `M ${points.join(' L ')}`
})

const areaPath = computed(() => {
  if (chartData.value.length === 0) return ''

  const line = linePath.value
  const width = chartData.value.length * 20
  return `${line} L ${width},100 L 0,100 Z`
})

// Methods
function getMetricValue(point: ChartDataPoint): number {
  switch (selectedMetric.value) {
    case 'trades': return point.trades
    case 'whales': return point.whales
    case 'average': return point.volume / Math.max(1, point.trades)
    default: return point.volume
  }
}

function getMetricUnit(): string {
  switch (selectedMetric.value) {
    case 'trades': return 'trades'
    case 'whales': return 'whales'
    case 'average': return 'per trade'
    default: return 'volume'
  }
}

function getBarHeight(point: ChartDataPoint): number {
  const value = getMetricValue(point)
  return maxValue.value > 0 ? Math.max(2, (value / maxValue.value) * 100) : 2
}

function getBarClass(point: ChartDataPoint, index: number): string {
  const baseClass = 'bg-gradient-to-t transition-all duration-200'

  // Color based on trend
  const prevValue = index > 0 ? getMetricValue(chartData.value[index - 1]) : getMetricValue(point)
  const currentValue = getMetricValue(point)

  if (currentValue > prevValue) {
    return `${baseClass} from-green-500/60 to-green-400/80 hover:from-green-500/80 hover:to-green-400`
  } else if (currentValue < prevValue) {
    return `${baseClass} from-red-500/60 to-red-400/80 hover:from-red-500/80 hover:to-red-400`
  } else {
    return `${baseClass} from-blue-500/60 to-blue-400/80 hover:from-blue-500/80 hover:to-blue-400`
  }
}

function getPointClass(point: ChartDataPoint, index: number): string {
  const prevValue = index > 0 ? getMetricValue(chartData.value[index - 1]) : getMetricValue(point)
  const currentValue = getMetricValue(point)

  if (currentValue > prevValue) {
    return 'bg-green-400 hover:bg-green-300'
  } else if (currentValue < prevValue) {
    return 'bg-red-400 hover:bg-red-300'
  } else {
    return 'bg-blue-400 hover:bg-blue-300'
  }
}

function formatCurrency(amount: number): string {
  if (amount >= 1e9) return `${(amount / 1e9).toFixed(2)}B`
  if (amount >= 1e6) return `${(amount / 1e6).toFixed(2)}M`
  if (amount >= 1e3) return `${(amount / 1e3).toFixed(2)}K`
  return amount.toFixed(2)
}

function formatTimeLabel(timestamp: string): string {
  const date = new Date(timestamp)

  if (props.timeRange === '1h') {
    return date.toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit'
    })
  } else if (props.timeRange === '24h') {
    return date.toLocaleTimeString('en-US', {
      hour: '2-digit'
    })
  } else {
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric'
    })
  }
}

function showTooltip(point: ChartDataPoint, index: number, event: MouseEvent) {
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect()

  tooltip.value = {
    show: true,
    x: rect.left + rect.width / 2,
    y: rect.top - 10,
    value: getMetricValue(point),
    time: formatTimeLabel(point.timestamp),
    trades: point.trades,
    whales: point.whales
  }
}

function hideTooltip() {
  tooltip.value.show = false
}

function zoomIn() {
  zoomLevel.value = Math.min(5, zoomLevel.value * 1.5)
}

function zoomOut() {
  zoomLevel.value = Math.max(1, zoomLevel.value / 1.5)
}

function resetZoom() {
  zoomLevel.value = 1
}

// Watch for loading state changes
watch(() => props.loading, (newLoading) => {
  isLoading.value = newLoading
})
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

/* Chart hover effects */
.group:hover .opacity-60 {
  opacity: 1;
}

/* Smooth transitions for chart elements */
.transition-all {
  transition: all 0.2s ease;
}

/* Custom grid styling */
.grid-rows-4 > div:last-child {
  border-bottom: none;
}
</style>