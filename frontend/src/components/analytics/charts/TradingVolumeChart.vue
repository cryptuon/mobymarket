<template>
  <Card variant="glass">
    <template #header>
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center space-x-3">
          <HeroIcon name="CurrencyDollarIcon" class="w-5 h-5 text-green-400" />
          <div>
            <h3 class="text-lg font-semibold text-white">Trading Volume</h3>
            <p class="text-xs text-white/60">Volume analysis and trends</p>
          </div>
        </div>

        <div class="flex items-center space-x-2">
          <select
            v-model="selectedTimeframe"
            class="bg-slate-800/50 border border-slate-600/50 rounded-lg px-3 py-1 text-white text-xs focus:outline-none focus:border-moby-500/50"
          >
            <option value="hourly">Hourly</option>
            <option value="daily">Daily</option>
            <option value="weekly">Weekly</option>
          </select>

          <div class="flex bg-slate-800/50 rounded-lg p-1">
            <button
              v-for="view in viewOptions"
              :key="view.value"
              @click="selectedView = view.value"
              :class="[
                'px-2 py-1 text-xs rounded transition-all',
                selectedView === view.value
                  ? 'bg-moby-500 text-white'
                  : 'text-white/60 hover:text-white'
              ]"
            >
              {{ view.label }}
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
        <div class="grid grid-cols-4 gap-4">
          <div v-for="i in 4" :key="i" class="animate-pulse">
            <div class="h-16 bg-slate-700/30 rounded-lg"></div>
          </div>
        </div>
      </div>

      <!-- Chart Container -->
      <div v-else class="relative">
        <!-- Volume Chart -->
        <div v-if="selectedView === 'volume'" class="h-64 bg-slate-800/20 rounded-lg p-4">
          <svg class="w-full h-full" viewBox="0 0 800 240">
            <!-- Grid Lines -->
            <defs>
              <pattern id="volumeGrid" width="40" height="24" patternUnits="userSpaceOnUse">
                <path d="M 40 0 L 0 0 0 24" fill="none" stroke="rgba(255,255,255,0.1)" stroke-width="0.5"/>
              </pattern>
            </defs>
            <rect width="100%" height="100%" fill="url(#volumeGrid)" />

            <!-- Y-axis labels -->
            <g class="text-xs fill-white/60">
              <text x="10" y="20">{{ formatVolume(maxVolume) }}</text>
              <text x="10" y="80">{{ formatVolume(maxVolume * 0.75) }}</text>
              <text x="10" y="140">{{ formatVolume(maxVolume * 0.5) }}</text>
              <text x="10" y="200">{{ formatVolume(maxVolume * 0.25) }}</text>
              <text x="10" y="235">0</text>
            </g>

            <!-- Volume Bars -->
            <g v-for="(bar, index) in volumeBars" :key="index">
              <rect
                :x="bar.x"
                :y="bar.y"
                :width="bar.width"
                :height="bar.height"
                :fill="bar.color"
                class="opacity-80 hover:opacity-100 transition-opacity cursor-pointer"
                @mouseenter="showVolumeTooltip(bar, index)"
                @mouseleave="hideTooltip"
              />
            </g>

            <!-- Volume Trend Line -->
            <path
              :d="volumeTrendPath"
              fill="none"
              stroke="#f59e0b"
              stroke-width="2"
              stroke-dasharray="3,3"
              class="opacity-60"
            />
          </svg>
        </div>

        <!-- Buy/Sell Distribution -->
        <div v-if="selectedView === 'buysell'" class="h-64 bg-slate-800/20 rounded-lg p-4">
          <svg class="w-full h-full" viewBox="0 0 800 240">
            <rect width="100%" height="100%" fill="url(#volumeGrid)" />

            <!-- Stacked bars for buy/sell volume -->
            <g v-for="(bar, index) in buySellBars" :key="index">
              <!-- Buy volume (bottom) -->
              <rect
                :x="bar.x"
                :y="bar.buyY"
                :width="bar.width"
                :height="bar.buyHeight"
                fill="#10b981"
                class="opacity-80 hover:opacity-100 transition-opacity cursor-pointer"
                @mouseenter="showBuySellTooltip(bar, index, 'buy')"
                @mouseleave="hideTooltip"
              />
              <!-- Sell volume (top) -->
              <rect
                :x="bar.x"
                :y="bar.sellY"
                :width="bar.width"
                :height="bar.sellHeight"
                fill="#ef4444"
                class="opacity-80 hover:opacity-100 transition-opacity cursor-pointer"
                @mouseenter="showBuySellTooltip(bar, index, 'sell')"
                @mouseleave="hideTooltip"
              />
            </g>
          </svg>

          <!-- Buy/Sell Legend -->
          <div class="flex items-center justify-center space-x-6 mt-4">
            <div class="flex items-center space-x-2">
              <div class="w-3 h-3 bg-green-500 rounded"></div>
              <span class="text-xs text-white/70">Buy Volume</span>
            </div>
            <div class="flex items-center space-x-2">
              <div class="w-3 h-3 bg-red-500 rounded"></div>
              <span class="text-xs text-white/70">Sell Volume</span>
            </div>
          </div>
        </div>

        <!-- Asset Volume Breakdown -->
        <div v-if="selectedView === 'assets'" class="h-64 bg-slate-800/20 rounded-lg p-4">
          <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 h-full">
            <!-- Asset Volume Chart -->
            <div class="relative">
              <svg class="w-full h-full" viewBox="0 0 200 200">
                <g v-for="(segment, index) in assetVolumeSegments" :key="index">
                  <path
                    :d="segment.path"
                    :fill="segment.color"
                    class="opacity-80 hover:opacity-100 transition-opacity cursor-pointer"
                    @mouseenter="showAssetTooltip(segment)"
                    @mouseleave="hideTooltip"
                  />
                </g>
                <!-- Center text -->
                <text x="100" y="95" text-anchor="middle" class="text-xs fill-white/60">Total Volume</text>
                <text x="100" y="110" text-anchor="middle" class="text-sm font-bold fill-white">
                  ${{ formatVolume(totalVolume) }}
                </text>
              </svg>
            </div>

            <!-- Asset Volume List -->
            <div class="space-y-2 overflow-y-auto">
              <div
                v-for="asset in assetVolumeData"
                :key="asset.symbol"
                class="flex items-center justify-between p-2 bg-slate-700/30 rounded-lg"
              >
                <div class="flex items-center space-x-2">
                  <div :class="asset.colorClass" class="w-3 h-3 rounded-full"></div>
                  <span class="text-sm text-white font-medium">{{ asset.symbol }}</span>
                </div>
                <div class="text-right">
                  <div class="text-sm font-bold text-white">${{ formatVolume(asset.volume) }}</div>
                  <div class="text-xs text-white/60">{{ asset.percentage.toFixed(1) }}%</div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Tooltip -->
        <div
          v-if="tooltip.show"
          :style="{ left: tooltip.x + 'px', top: tooltip.y + 'px' }"
          class="absolute z-10 bg-slate-800/90 backdrop-blur border border-white/20 rounded-lg p-3 text-xs pointer-events-none"
        >
          <div class="font-semibold text-white">{{ tooltip.label }}</div>
          <div class="text-white/70">{{ tooltip.time }}</div>
          <div class="font-bold text-white">${{ formatVolume(tooltip.value) }}</div>
          <div v-if="tooltip.extra" class="text-white/60 mt-1">{{ tooltip.extra }}</div>
        </div>
      </div>

      <!-- Volume Statistics -->
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <div class="bg-slate-800/30 rounded-lg p-4 text-center">
          <div class="flex items-center justify-center space-x-2 mb-2">
            <HeroIcon name="CurrencyDollarIcon" class="w-4 h-4 text-green-400" />
            <span class="text-xs text-white/60">24h Volume</span>
          </div>
          <div class="text-xl font-bold text-white">${{ formatVolume(volumeStats.volume24h) }}</div>
          <div :class="['text-xs', getChangeColor(volumeStats.volume24hChange)]">
            {{ volumeStats.volume24hChange >= 0 ? '+' : '' }}{{ volumeStats.volume24hChange.toFixed(1) }}%
          </div>
        </div>

        <div class="bg-slate-800/30 rounded-lg p-4 text-center">
          <div class="flex items-center justify-center space-x-2 mb-2">
            <HeroIcon name="ArrowTrendingUpIcon" class="w-4 h-4 text-blue-400" />
            <span class="text-xs text-white/60">Avg Daily</span>
          </div>
          <div class="text-xl font-bold text-white">${{ formatVolume(volumeStats.avgDaily) }}</div>
          <div class="text-xs text-white/60">7-day average</div>
        </div>

        <div class="bg-slate-800/30 rounded-lg p-4 text-center">
          <div class="flex items-center justify-center space-x-2 mb-2">
            <HeroIcon name="ScaleIcon" class="w-4 h-4 text-purple-400" />
            <span class="text-xs text-white/60">Buy/Sell Ratio</span>
          </div>
          <div :class="['text-xl font-bold', getBuySellRatioColor(volumeStats.buySellRatio)]">
            {{ volumeStats.buySellRatio.toFixed(2) }}
          </div>
          <div class="text-xs text-white/60">{{ getBuySellRatioLabel(volumeStats.buySellRatio) }}</div>
        </div>

        <div class="bg-slate-800/30 rounded-lg p-4 text-center">
          <div class="flex items-center justify-center space-x-2 mb-2">
            <HeroIcon name="ClockIcon" class="w-4 h-4 text-yellow-400" />
            <span class="text-xs text-white/60">Peak Hour</span>
          </div>
          <div class="text-xl font-bold text-white">{{ volumeStats.peakHour }}:00</div>
          <div class="text-xs text-white/60">UTC</div>
        </div>
      </div>

      <!-- Volume Alerts -->
      <div v-if="volumeAlerts.length > 0" class="space-y-2">
        <h4 class="text-sm font-semibold text-white flex items-center space-x-2">
          <HeroIcon name="BellIcon" class="w-4 h-4 text-yellow-400" />
          <span>Volume Alerts</span>
        </h4>
        <div class="space-y-2">
          <div
            v-for="alert in volumeAlerts"
            :key="alert.id"
            :class="getAlertClass(alert.type)"
            class="p-3 rounded-lg border"
          >
            <div class="flex items-start space-x-2">
              <HeroIcon :name="getAlertIcon(alert.type)" class="w-4 h-4 mt-0.5 flex-shrink-0" />
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

interface VolumeData {
  timestamp: string
  volume: number
  buyVolume: number
  sellVolume: number
  assets: Record<string, number>
}

interface VolumeAlert {
  id: string
  title: string
  message: string
  type: 'spike' | 'drop' | 'unusual'
}

interface Props {
  data: VolumeData[]
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

const selectedTimeframe = ref('daily')
const selectedView = ref('volume')

const viewOptions = [
  { label: 'Volume', value: 'volume' },
  { label: 'Buy/Sell', value: 'buysell' },
  { label: 'Assets', value: 'assets' }
]

const tooltip = ref({
  show: false,
  x: 0,
  y: 0,
  label: '',
  time: '',
  value: 0,
  extra: ''
})

// Generate mock data if none provided
const generateMockVolumeData = (): VolumeData[] => {
  const data: VolumeData[] = []
  const timeframes: Record<string, { count: number, interval: number }> = {
    hourly: { count: 24, interval: 1 },
    daily: { count: 30, interval: 24 },
    weekly: { count: 12, interval: 168 }
  }

  const config = timeframes[selectedTimeframe.value]
  let baseVolume = 1000000

  for (let i = 0; i < config.count; i++) {
    const date = new Date()
    date.setHours(date.getHours() - (config.count - i - 1) * config.interval)

    const volumeVariation = 0.7 + Math.random() * 0.6
    const volume = baseVolume * volumeVariation
    const buyRatio = 0.3 + Math.random() * 0.4
    const buyVolume = volume * buyRatio
    const sellVolume = volume * (1 - buyRatio)

    data.push({
      timestamp: date.toISOString(),
      volume,
      buyVolume,
      sellVolume,
      assets: {
        ETH: volume * (0.3 + Math.random() * 0.2),
        BTC: volume * (0.2 + Math.random() * 0.15),
        UNI: volume * (0.1 + Math.random() * 0.1),
        AAVE: volume * (0.05 + Math.random() * 0.1),
        COMP: volume * (0.03 + Math.random() * 0.07),
        Other: volume * (0.1 + Math.random() * 0.1)
      }
    })
  }

  return data
}

const volumeData = computed(() => props.data.length ? props.data : generateMockVolumeData())

const maxVolume = computed(() => {
  return Math.max(...volumeData.value.map(d => d.volume))
})

const totalVolume = computed(() => {
  return volumeData.value.reduce((sum, d) => sum + d.volume, 0)
})

// Volume bars for chart
const volumeBars = computed(() => {
  const barWidth = 760 / volumeData.value.length
  const padding = barWidth * 0.2

  return volumeData.value.map((d, index) => {
    const height = (d.volume / maxVolume.value) * 200
    const x = 20 + index * barWidth + padding / 2
    const y = 220 - height

    return {
      x,
      y,
      width: barWidth - padding,
      height,
      color: d.volume > totalVolume.value / volumeData.value.length ? '#10b981' : '#60a5fa',
      volume: d.volume,
      timestamp: d.timestamp
    }
  })
})

// Volume trend line
const volumeTrendPath = computed(() => {
  const points = volumeData.value.map((d, index) => {
    const x = 20 + (index / (volumeData.value.length - 1)) * 760
    const y = 220 - (d.volume / maxVolume.value) * 200
    return `${index === 0 ? 'M' : 'L'} ${x} ${y}`
  })
  return points.join(' ')
})

// Buy/Sell bars
const buySellBars = computed(() => {
  const barWidth = 760 / volumeData.value.length
  const padding = barWidth * 0.2

  return volumeData.value.map((d, index) => {
    const totalHeight = (d.volume / maxVolume.value) * 200
    const buyHeight = (d.buyVolume / d.volume) * totalHeight
    const sellHeight = (d.sellVolume / d.volume) * totalHeight

    const x = 20 + index * barWidth + padding / 2
    const buyY = 220 - buyHeight
    const sellY = buyY - sellHeight

    return {
      x,
      buyY,
      sellY,
      width: barWidth - padding,
      buyHeight,
      sellHeight,
      buyVolume: d.buyVolume,
      sellVolume: d.sellVolume,
      timestamp: d.timestamp
    }
  })
})

// Asset volume data
const assetVolumeData = computed(() => {
  const latest = volumeData.value[volumeData.value.length - 1]
  if (!latest) return []

  const assets = Object.entries(latest.assets).map(([symbol, volume]) => ({
    symbol,
    volume,
    percentage: (volume / latest.volume) * 100,
    colorClass: getAssetColor(symbol)
  }))

  return assets.sort((a, b) => b.volume - a.volume)
})

// Asset volume pie chart segments
const assetVolumeSegments = computed(() => {
  let currentAngle = 0
  const radius = 80
  const centerX = 100
  const centerY = 100

  return assetVolumeData.value.map(asset => {
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
      color: getAssetColorHex(asset.symbol),
      symbol: asset.symbol,
      volume: asset.volume,
      percentage: asset.percentage
    }
  })
})

// Volume statistics
const volumeStats = computed(() => {
  if (volumeData.value.length === 0) {
    return {
      volume24h: 0,
      volume24hChange: 0,
      avgDaily: 0,
      buySellRatio: 1,
      peakHour: 0
    }
  }

  const latest = volumeData.value[volumeData.value.length - 1]
  const previous = volumeData.value[volumeData.value.length - 2]

  const volume24h = latest.volume
  const volume24hChange = previous ? ((latest.volume - previous.volume) / previous.volume) * 100 : 0
  const avgDaily = volumeData.value.reduce((sum, d) => sum + d.volume, 0) / volumeData.value.length
  const buySellRatio = latest.buyVolume / latest.sellVolume

  // Find peak hour (simplified)
  const hourlyVolumes = Array(24).fill(0)
  volumeData.value.forEach(d => {
    const hour = new Date(d.timestamp).getHours()
    hourlyVolumes[hour] += d.volume
  })
  const peakHour = hourlyVolumes.indexOf(Math.max(...hourlyVolumes))

  return {
    volume24h,
    volume24hChange,
    avgDaily,
    buySellRatio,
    peakHour
  }
})

// Volume alerts
const volumeAlerts = computed(() => {
  const alerts: VolumeAlert[] = []
  const avg = volumeStats.value.avgDaily
  const latest = volumeData.value[volumeData.value.length - 1]

  if (latest && latest.volume > avg * 2) {
    alerts.push({
      id: '1',
      title: 'Volume Spike Detected',
      message: `Current volume is ${((latest.volume / avg) * 100).toFixed(0)}% above average`,
      type: 'spike'
    })
  }

  if (latest && latest.volume < avg * 0.5) {
    alerts.push({
      id: '2',
      title: 'Low Volume Alert',
      message: `Current volume is ${(100 - (latest.volume / avg) * 100).toFixed(0)}% below average`,
      type: 'drop'
    })
  }

  if (volumeStats.value.buySellRatio > 2 || volumeStats.value.buySellRatio < 0.5) {
    alerts.push({
      id: '3',
      title: 'Unusual Buy/Sell Activity',
      message: `Buy/sell ratio at ${volumeStats.value.buySellRatio.toFixed(2)} indicates market imbalance`,
      type: 'unusual'
    })
  }

  return alerts
})

// Methods
function formatVolume(volume: number): string {
  if (volume >= 1e9) return `${(volume / 1e9).toFixed(1)}B`
  if (volume >= 1e6) return `${(volume / 1e6).toFixed(1)}M`
  if (volume >= 1e3) return `${(volume / 1e3).toFixed(1)}K`
  return volume.toFixed(0)
}

function getAssetColor(symbol: string): string {
  const colors: Record<string, string> = {
    ETH: 'bg-purple-400',
    BTC: 'bg-orange-400',
    UNI: 'bg-pink-400',
    AAVE: 'bg-blue-400',
    COMP: 'bg-green-400',
    Other: 'bg-gray-400'
  }
  return colors[symbol] || 'bg-gray-400'
}

function getAssetColorHex(symbol: string): string {
  const colors: Record<string, string> = {
    ETH: '#a855f7',
    BTC: '#fb923c',
    UNI: '#f472b6',
    AAVE: '#60a5fa',
    COMP: '#4ade80',
    Other: '#9ca3af'
  }
  return colors[symbol] || '#9ca3af'
}

function showVolumeTooltip(bar: any, index: number) {
  tooltip.value = {
    show: true,
    x: bar.x + bar.width / 2,
    y: bar.y - 10,
    label: 'Trading Volume',
    time: new Date(bar.timestamp).toLocaleString(),
    value: bar.volume,
    extra: ''
  }
}

function showBuySellTooltip(bar: any, index: number, type: 'buy' | 'sell') {
  const volume = type === 'buy' ? bar.buyVolume : bar.sellVolume
  tooltip.value = {
    show: true,
    x: bar.x + bar.width / 2,
    y: type === 'buy' ? bar.buyY - 10 : bar.sellY - 10,
    label: `${type === 'buy' ? 'Buy' : 'Sell'} Volume`,
    time: new Date(bar.timestamp).toLocaleString(),
    value: volume,
    extra: `${((volume / (bar.buyVolume + bar.sellVolume)) * 100).toFixed(1)}% of total`
  }
}

function showAssetTooltip(segment: any) {
  tooltip.value = {
    show: true,
    x: 200,
    y: 100,
    label: segment.symbol,
    time: '',
    value: segment.volume,
    extra: `${segment.percentage.toFixed(1)}% of total volume`
  }
}

function hideTooltip() {
  tooltip.value.show = false
}

function getChangeColor(change: number): string {
  return change >= 0 ? 'text-green-400' : 'text-red-400'
}

function getBuySellRatioColor(ratio: number): string {
  if (ratio > 1.5) return 'text-green-400'
  if (ratio < 0.67) return 'text-red-400'
  return 'text-yellow-400'
}

function getBuySellRatioLabel(ratio: number): string {
  if (ratio > 1.5) return 'Buy pressure'
  if (ratio < 0.67) return 'Sell pressure'
  return 'Balanced'
}

function getAlertClass(type: string): string {
  switch (type) {
    case 'spike':
      return 'bg-green-500/10 border-green-500/30 text-green-400'
    case 'drop':
      return 'bg-red-500/10 border-red-500/30 text-red-400'
    default:
      return 'bg-yellow-500/10 border-yellow-500/30 text-yellow-400'
  }
}

function getAlertIcon(type: string): string {
  switch (type) {
    case 'spike':
      return 'ArrowTrendingUpIcon'
    case 'drop':
      return 'ArrowTrendingDownIcon'
    default:
      return 'ExclamationTriangleIcon'
  }
}
</script>

<style scoped>
/* Chart animations */
rect {
  transition: all 0.3s ease;
}

path {
  transition: all 0.3s ease;
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