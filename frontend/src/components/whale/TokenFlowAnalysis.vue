<template>
  <Card variant="glass">
    <template #header>
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center space-x-3">
          <HeroIcon name="ArrowsRightLeftIcon" class="w-5 h-5 text-purple-400" />
          <div>
            <h3 class="text-lg font-semibold text-white">Token Flow Analysis</h3>
            <p class="text-xs text-white/60">Whale capital movements</p>
          </div>
        </div>

        <div class="flex items-center space-x-2">
          <!-- Flow Direction Filter -->
          <select
            v-model="flowFilter"
            class="bg-slate-800/50 border border-slate-600/50 rounded-lg px-3 py-1 text-white text-xs focus:outline-none focus:border-moby-500/50"
          >
            <option value="all">All Flows</option>
            <option value="inflow">Inflows Only</option>
            <option value="outflow">Outflows Only</option>
            <option value="net_positive">Net Positive</option>
            <option value="net_negative">Net Negative</option>
          </select>

          <!-- Refresh Button -->
          <button
            @click="refreshData"
            :disabled="isLoading"
            class="p-2 hover:bg-white/10 rounded-lg transition-colors disabled:opacity-50"
          >
            <HeroIcon
              name="ArrowPathIcon"
              class="w-4 h-4 text-white/70"
              :class="{ 'animate-spin': isLoading }"
            />
          </button>
        </div>
      </div>
    </template>

    <div class="space-y-4">
      <!-- Flow Summary Cards -->
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div class="bg-green-500/10 border border-green-500/30 rounded-lg p-3">
          <div class="flex items-center space-x-2 mb-1">
            <HeroIcon name="ArrowDownIcon" class="w-4 h-4 text-green-400" />
            <span class="text-xs text-green-400 font-medium">Total Inflows</span>
          </div>
          <div class="text-xl font-bold text-white">${{ formatCurrency(totalInflows) }}</div>
          <div class="text-xs text-green-400/80">+{{ inflowPercentage.toFixed(1) }}% vs yesterday</div>
        </div>

        <div class="bg-red-500/10 border border-red-500/30 rounded-lg p-3">
          <div class="flex items-center space-x-2 mb-1">
            <HeroIcon name="ArrowUpIcon" class="w-4 h-4 text-red-400" />
            <span class="text-xs text-red-400 font-medium">Total Outflows</span>
          </div>
          <div class="text-xl font-bold text-white">${{ formatCurrency(totalOutflows) }}</div>
          <div class="text-xs text-red-400/80">+{{ outflowPercentage.toFixed(1) }}% vs yesterday</div>
        </div>

        <div class="bg-blue-500/10 border border-blue-500/30 rounded-lg p-3">
          <div class="flex items-center space-x-2 mb-1">
            <HeroIcon name="ScaleIcon" class="w-4 h-4 text-blue-400" />
            <span class="text-xs text-blue-400 font-medium">Net Flow</span>
          </div>
          <div class="text-xl font-bold text-white">
            <span :class="netFlowColorClass">
              {{ netFlow >= 0 ? '+' : '' }}${{ formatCurrency(Math.abs(netFlow)) }}
            </span>
          </div>
          <div class="text-xs text-blue-400/80">{{ Math.abs(netFlowPercentage).toFixed(1) }}% net {{ netFlow >= 0 ? 'inflow' : 'outflow' }}</div>
        </div>
      </div>

      <!-- Token Flow List -->
      <div class="space-y-3">
        <div
          v-for="flow in filteredFlows"
          :key="flow.token"
          class="bg-slate-800/30 hover:bg-slate-700/30 border border-slate-600/30 hover:border-slate-500/50 rounded-xl p-4 transition-all duration-200 cursor-pointer group"
          @click="selectToken(flow)"
        >
          <div class="flex items-center justify-between">
            <!-- Token Info -->
            <div class="flex items-center space-x-3 flex-1">
              <!-- Token Icon -->
              <div class="relative">
                <img
                  :src="getTokenIcon(flow.token)"
                  :alt="flow.token"
                  class="w-10 h-10 rounded-full"
                  @error="handleImageError"
                />
                <!-- Flow Direction Indicator -->
                <div
                  :class="getFlowIndicatorClass(flow.netFlow)"
                  class="absolute -bottom-1 -right-1 w-5 h-5 rounded-full border-2 border-slate-800 flex items-center justify-center"
                >
                  <HeroIcon
                    :name="flow.netFlow >= 0 ? 'ArrowDownIcon' : 'ArrowUpIcon'"
                    class="w-3 h-3"
                  />
                </div>
              </div>

              <!-- Token Details -->
              <div class="flex-1">
                <div class="flex items-center space-x-2 mb-1">
                  <span class="text-white font-semibold">{{ flow.token }}</span>
                  <span
                    :class="getFlowBadgeClass(flow.netFlow)"
                    class="px-2 py-1 rounded-full text-xs font-medium"
                  >
                    {{ flow.netFlow >= 0 ? 'Net Inflow' : 'Net Outflow' }}
                  </span>
                </div>

                <!-- Flow Metrics -->
                <div class="grid grid-cols-2 lg:grid-cols-4 gap-4 mt-2">
                  <div>
                    <div class="text-xs text-white/60">Inflow</div>
                    <div class="text-sm font-medium text-green-400">${{ formatCurrency(flow.inflowVolume) }}</div>
                  </div>
                  <div>
                    <div class="text-xs text-white/60">Outflow</div>
                    <div class="text-sm font-medium text-red-400">${{ formatCurrency(flow.outflowVolume) }}</div>
                  </div>
                  <div>
                    <div class="text-xs text-white/60">Net Flow</div>
                    <div :class="['text-sm font-medium', getNetFlowColor(flow.netFlow)]">
                      {{ flow.netFlow >= 0 ? '+' : '' }}${{ formatCurrency(Math.abs(flow.netFlow)) }}
                    </div>
                  </div>
                  <div>
                    <div class="text-xs text-white/60">24h Change</div>
                    <div :class="['text-sm font-medium', getChangeColor(flow.change24h)]">
                      {{ flow.change24h >= 0 ? '+' : '' }}{{ flow.change24h.toFixed(1) }}%
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Flow Visualization -->
            <div class="flex-shrink-0 ml-4">
              <div class="w-24 h-8 bg-slate-700/50 rounded-lg overflow-hidden relative">
                <!-- Inflow Bar -->
                <div
                  class="absolute left-0 top-0 h-full bg-green-500/60 transition-all duration-500"
                  :style="{ width: `${getInflowPercentage(flow)}%` }"
                ></div>
                <!-- Outflow Bar -->
                <div
                  class="absolute right-0 top-0 h-full bg-red-500/60 transition-all duration-500"
                  :style="{ width: `${getOutflowPercentage(flow)}%` }"
                ></div>
                <!-- Net Flow Indicator -->
                <div
                  class="absolute top-1/2 transform -translate-y-1/2 w-0.5 h-6 bg-white transition-all duration-500"
                  :style="{ left: `${getNetFlowPosition(flow)}%` }"
                ></div>
              </div>
              <div class="text-xs text-white/60 text-center mt-1">Flow Balance</div>
            </div>
          </div>

          <!-- Expanded Details (on hover/click) -->
          <Transition
            name="details"
            enter-active-class="transition-all duration-200"
            enter-from-class="opacity-0 max-h-0"
            enter-to-class="opacity-100 max-h-32"
            leave-active-class="transition-all duration-150"
            leave-from-class="opacity-100 max-h-32"
            leave-to-class="opacity-0 max-h-0"
          >
            <div v-if="expandedToken === flow.token" class="mt-4 pt-4 border-t border-white/10">
              <div class="grid grid-cols-2 lg:grid-cols-4 gap-4 text-sm">
                <div>
                  <div class="text-xs text-white/60">Whale Count</div>
                  <div class="text-white font-medium">{{ flow.whaleCount || 'N/A' }}</div>
                </div>
                <div>
                  <div class="text-xs text-white/60">Avg Trade Size</div>
                  <div class="text-white font-medium">${{ formatCurrency(flow.avgTradeSize || 0) }}</div>
                </div>
                <div>
                  <div class="text-xs text-white/60">Top Exchange</div>
                  <div class="text-white font-medium">{{ flow.topExchange || 'Various' }}</div>
                </div>
                <div>
                  <div class="text-xs text-white/60">Flow Trend</div>
                  <div :class="['font-medium', getTrendColor(flow.trend)]">
                    {{ flow.trend || 'Stable' }}
                  </div>
                </div>
              </div>
            </div>
          </Transition>
        </div>
      </div>

      <!-- Empty State -->
      <div v-if="filteredFlows.length === 0" class="text-center py-8">
        <HeroIcon name="MagnifyingGlassIcon" class="w-12 h-12 text-white/30 mx-auto mb-2" />
        <p class="text-white/60 text-sm">No token flows match your current filter</p>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center justify-between text-xs text-white/50">
        <span>Last updated: {{ formatLastUpdate(lastUpdate) }}</span>
        <div class="flex items-center space-x-2">
          <div class="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>
          <span>Real-time data</span>
        </div>
      </div>
    </template>
  </Card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

import Card from '@components/ui/Card.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'

interface TokenFlow {
  token: string
  netFlow: number
  inflowVolume: number
  outflowVolume: number
  change24h: number
  whaleCount?: number
  avgTradeSize?: number
  topExchange?: string
  trend?: string
}

interface Props {
  flows: TokenFlow[]
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'token-selected': [flow: TokenFlow]
  'refresh': []
}>()

const flowFilter = ref('all')
const expandedToken = ref<string | null>(null)
const isLoading = ref(false)
const lastUpdate = ref(new Date().toISOString())

// Computed properties
const totalInflows = computed(() => {
  return props.flows.reduce((sum, flow) => sum + flow.inflowVolume, 0)
})

const totalOutflows = computed(() => {
  return props.flows.reduce((sum, flow) => sum + flow.outflowVolume, 0)
})

const netFlow = computed(() => totalInflows.value - totalOutflows.value)

const netFlowColorClass = computed(() => {
  return netFlow.value >= 0 ? 'text-green-400' : 'text-red-400'
})

const inflowPercentage = computed(() => Math.random() * 20 + 5) // Mock data
const outflowPercentage = computed(() => Math.random() * 15 + 3) // Mock data
const netFlowPercentage = computed(() => (netFlow.value / totalInflows.value) * 100)

const filteredFlows = computed(() => {
  return props.flows.filter(flow => {
    switch (flowFilter.value) {
      case 'inflow':
        return flow.inflowVolume > flow.outflowVolume
      case 'outflow':
        return flow.outflowVolume > flow.inflowVolume
      case 'net_positive':
        return flow.netFlow > 0
      case 'net_negative':
        return flow.netFlow < 0
      default:
        return true
    }
  })
})

// Methods
function formatCurrency(amount: number): string {
  if (amount >= 1e9) return `${(amount / 1e9).toFixed(2)}B`
  if (amount >= 1e6) return `${(amount / 1e6).toFixed(2)}M`
  if (amount >= 1e3) return `${(amount / 1e3).toFixed(2)}K`
  return amount.toFixed(2)
}

function getTokenIcon(symbol: string): string {
  const iconMap: Record<string, string> = {
    ETH: '/tokens/eth.svg',
    BTC: '/tokens/btc.svg',
    USDC: '/tokens/usdc.svg',
    USDT: '/tokens/usdt.svg',
    DAI: '/tokens/dai.svg'
  }
  return iconMap[symbol] || '/tokens/default.svg'
}

function handleImageError(event: Event) {
  const img = event.target as HTMLImageElement
  img.src = '/tokens/default.svg'
}

function getFlowIndicatorClass(netFlow: number): string {
  return netFlow >= 0
    ? 'bg-green-500 text-white'
    : 'bg-red-500 text-white'
}

function getFlowBadgeClass(netFlow: number): string {
  return netFlow >= 0
    ? 'bg-green-500/20 text-green-400'
    : 'bg-red-500/20 text-red-400'
}

function getNetFlowColor(netFlow: number): string {
  return netFlow >= 0 ? 'text-green-400' : 'text-red-400'
}

function getChangeColor(change: number): string {
  return change >= 0 ? 'text-green-400' : 'text-red-400'
}

function getTrendColor(trend?: string): string {
  switch (trend?.toLowerCase()) {
    case 'bullish':
    case 'strong inflow':
      return 'text-green-400'
    case 'bearish':
    case 'strong outflow':
      return 'text-red-400'
    default:
      return 'text-white/70'
  }
}

function getInflowPercentage(flow: TokenFlow): number {
  const total = flow.inflowVolume + flow.outflowVolume
  return total > 0 ? (flow.inflowVolume / total) * 100 : 0
}

function getOutflowPercentage(flow: TokenFlow): number {
  const total = flow.inflowVolume + flow.outflowVolume
  return total > 0 ? (flow.outflowVolume / total) * 100 : 0
}

function getNetFlowPosition(flow: TokenFlow): number {
  const total = flow.inflowVolume + flow.outflowVolume
  if (total === 0) return 50

  // Position based on net flow relative to total volume
  const netRatio = flow.netFlow / total
  return 50 + (netRatio * 50) // 0-100% range centered at 50%
}

function selectToken(flow: TokenFlow) {
  expandedToken.value = expandedToken.value === flow.token ? null : flow.token
  emit('token-selected', flow)
}

async function refreshData() {
  if (isLoading.value) return

  isLoading.value = true
  try {
    emit('refresh')
    await new Promise(resolve => setTimeout(resolve, 1000))
    lastUpdate.value = new Date().toISOString()
  } finally {
    isLoading.value = false
  }
}

function formatLastUpdate(timestamp: string): string {
  const diff = Date.now() - new Date(timestamp).getTime()
  const seconds = Math.floor(diff / 1000)

  if (seconds < 60) return `${seconds}s ago`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`
  return `${Math.floor(seconds / 3600)}h ago`
}
</script>

<style scoped>
/* Smooth transitions for flow visualizations */
.transition-all {
  transition: all 0.3s ease;
}

/* Details expansion animation */
.details-enter-active,
.details-leave-active {
  transition: all 0.2s ease;
  overflow: hidden;
}

.details-enter-from,
.details-leave-to {
  opacity: 0;
  max-height: 0;
}

.details-enter-to,
.details-leave-from {
  opacity: 1;
  max-height: 8rem;
}

/* Flow visualization bars */
.flow-bar {
  transition: width 0.5s ease;
}

/* Hover effects */
.group:hover .opacity-70 {
  opacity: 1;
}
</style>