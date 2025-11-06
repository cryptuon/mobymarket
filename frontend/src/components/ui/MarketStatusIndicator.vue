<template>
  <div class="flex items-center space-x-3 bg-glass-light backdrop-blur-md border border-white/20 rounded-xl px-4 py-2">
    <!-- Market Status Indicator -->
    <div class="flex items-center space-x-2">
      <div :class="[
        'w-2 h-2 rounded-full',
        statusColor,
        isLive ? 'animate-pulse' : ''
      ]"></div>
      <span class="text-sm font-medium text-white">{{ statusText }}</span>
    </div>

    <!-- Market Stats -->
    <div class="hidden lg:flex items-center space-x-4 text-sm">
      <!-- Global Market Cap -->
      <div class="flex items-center space-x-1">
        <span class="text-white/60">MCap:</span>
        <span class="text-white font-medium">${{ formatMarketCap(globalMarketCap) }}</span>
        <span :class="[
          'text-xs',
          marketCapChange >= 0 ? 'text-green-400' : 'text-red-400'
        ]">
          {{ marketCapChange >= 0 ? '+' : '' }}{{ marketCapChange.toFixed(2) }}%
        </span>
      </div>

      <!-- Trading Volume -->
      <div class="flex items-center space-x-1">
        <span class="text-white/60">Vol:</span>
        <span class="text-white font-medium">${{ formatVolume(volume24h) }}</span>
      </div>

      <!-- Whale Activity -->
      <div class="flex items-center space-x-1">
        <HeroIcon name="EyeIcon" class="w-3 h-3 text-moby-400" />
        <span class="text-white/60">Whales:</span>
        <span class="text-moby-400 font-medium">{{ activeWhales }}</span>
      </div>

      <!-- Gas Price -->
      <div class="flex items-center space-x-1">
        <span class="text-white/60">Gas:</span>
        <span :class="[
          'font-medium',
          gasPrice <= 20 ? 'text-green-400' :
          gasPrice <= 50 ? 'text-yellow-400' : 'text-red-400'
        ]">
          {{ gasPrice }} gwei
        </span>
      </div>
    </div>

    <!-- Refresh Button -->
    <button
      @click="refreshData"
      :disabled="isRefreshing"
      class="p-1 hover:bg-white/10 rounded-lg transition-all"
      :class="{ 'animate-spin': isRefreshing }"
      aria-label="Refresh market data"
    >
      <HeroIcon name="ArrowPathIcon" class="w-4 h-4 text-white/70" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { storeToRefs } from 'pinia'

import HeroIcon from '@components/ui/HeroIcon.vue'

import { useMarketStore } from '@stores/market'

const marketStore = useMarketStore()

const {
  isConnected: isMarketConnected,
  globalMarketCap,
  marketCapChange,
  volume24h,
  gasPrice,
  activeWhales,
  lastUpdated
} = storeToRefs(marketStore)

const isRefreshing = ref(false)
const refreshInterval = ref<NodeJS.Timeout>()

// Computed properties
const isLive = computed(() => {
  if (!lastUpdated.value) return false
  const timeDiff = Date.now() - new Date(lastUpdated.value).getTime()
  return timeDiff < 30000 // Live if updated within last 30 seconds
})

const statusText = computed(() => {
  if (!isMarketConnected.value) return 'Disconnected'
  if (isLive.value) return 'Live'
  return 'Delayed'
})

const statusColor = computed(() => {
  if (!isMarketConnected.value) return 'bg-red-500'
  if (isLive.value) return 'bg-green-500'
  return 'bg-yellow-500'
})

// Methods
function formatMarketCap(value: number): string {
  if (value >= 1e12) return `${(value / 1e12).toFixed(2)}T`
  if (value >= 1e9) return `${(value / 1e9).toFixed(2)}B`
  if (value >= 1e6) return `${(value / 1e6).toFixed(2)}M`
  return value.toFixed(2)
}

function formatVolume(value: number): string {
  if (value >= 1e9) return `${(value / 1e9).toFixed(1)}B`
  if (value >= 1e6) return `${(value / 1e6).toFixed(1)}M`
  if (value >= 1e3) return `${(value / 1e3).toFixed(1)}K`
  return value.toFixed(0)
}

async function refreshData() {
  if (isRefreshing.value) return

  isRefreshing.value = true
  try {
    await marketStore.fetchMarketData()
  } catch (error) {
    console.error('Failed to refresh market data:', error)
  } finally {
    isRefreshing.value = false
  }
}

function startAutoRefresh() {
  refreshInterval.value = setInterval(() => {
    marketStore.fetchMarketData()
  }, 30000) // Refresh every 30 seconds
}

function stopAutoRefresh() {
  if (refreshInterval.value) {
    clearInterval(refreshInterval.value)
    refreshInterval.value = undefined
  }
}

// Lifecycle
onMounted(() => {
  // Initial data fetch
  marketStore.fetchMarketData()
  startAutoRefresh()
})

onUnmounted(() => {
  stopAutoRefresh()
})
</script>