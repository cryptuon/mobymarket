<template>
  <div class="bg-slate-900/50 backdrop-blur-sm border-b border-white/10 overflow-hidden">
    <div class="relative h-12 flex items-center">
      <!-- Scrolling Price Feed -->
      <div
        ref="tickerContainer"
        class="flex items-center space-x-8 animate-scroll whitespace-nowrap"
        :style="{ animationDuration: `${scrollDuration}s` }"
      >
        <!-- Repeat ticker items for seamless loop -->
        <div
          v-for="(prices, groupIndex) in [livePrices, livePrices]"
          :key="groupIndex"
          class="flex items-center space-x-8"
        >
          <div
            v-for="price in prices"
            :key="`${groupIndex}-${price.symbol}`"
            class="flex items-center space-x-3 px-4 py-2 bg-slate-800/30 rounded-lg cursor-pointer hover:bg-slate-700/30 transition-colors group"
            @click="selectToken(price.symbol)"
          >
            <!-- Token Icon -->
            <img
              :src="getTokenIcon(price.symbol)"
              :alt="price.symbol"
              class="w-6 h-6 rounded-full"
              @error="handleImageError"
            />

            <!-- Token Info -->
            <div class="flex items-center space-x-2">
              <span class="text-white font-semibold text-sm">{{ price.symbol }}</span>
              <span class="text-white text-sm">${{ formatPrice(price.price) }}</span>
              <span
                :class="[
                  'text-xs font-medium px-2 py-1 rounded-full',
                  price.change24h >= 0
                    ? 'bg-green-500/20 text-green-400'
                    : 'bg-red-500/20 text-red-400'
                ]"
              >
                {{ price.change24h >= 0 ? '+' : '' }}{{ price.change24h.toFixed(2) }}%
              </span>
            </div>

            <!-- Live indicator -->
            <div
              v-if="isRecentUpdate(price)"
              class="w-2 h-2 bg-green-400 rounded-full animate-pulse"
            ></div>
          </div>
        </div>
      </div>

      <!-- Gradient Overlays -->
      <div class="absolute left-0 top-0 bottom-0 w-16 bg-gradient-to-r from-slate-900/50 to-transparent pointer-events-none"></div>
      <div class="absolute right-0 top-0 bottom-0 w-16 bg-gradient-to-l from-slate-900/50 to-transparent pointer-events-none"></div>

      <!-- Connection Status -->
      <div class="absolute right-4 top-1/2 transform -translate-y-1/2">
        <div
          :class="connectionStatusClass"
          class="flex items-center space-x-1 px-2 py-1 rounded-lg text-xs font-medium"
        >
          <div :class="statusDotClass" class="w-2 h-2 rounded-full"></div>
          <span>{{ connectionStatusText }}</span>
        </div>
      </div>
    </div>

    <!-- Mobile-friendly horizontal scroll version -->
    <div class="sm:hidden px-4 py-2">
      <div class="flex space-x-4 overflow-x-auto scrollbar-hide">
        <div
          v-for="price in livePrices"
          :key="price.symbol"
          class="flex-shrink-0 flex items-center space-x-2 px-3 py-2 bg-slate-800/30 rounded-lg"
          @click="selectToken(price.symbol)"
        >
          <img
            :src="getTokenIcon(price.symbol)"
            :alt="price.symbol"
            class="w-5 h-5 rounded-full"
            @error="handleImageError"
          />
          <div class="text-white text-sm font-medium">{{ price.symbol }}</div>
          <div class="text-white text-sm">${{ formatPrice(price.price) }}</div>
          <div
            :class="[
              'text-xs px-1 py-0.5 rounded',
              price.change24h >= 0 ? 'text-green-400' : 'text-red-400'
            ]"
          >
            {{ price.change24h >= 0 ? '+' : '' }}{{ price.change24h.toFixed(1) }}%
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'

import { useRealTimeData } from '@/composables/useRealTimeData'
import { useTradingStore } from '@/stores/trading'
import type { TokenPrice } from '@/types'

const emit = defineEmits<{
  'token-selected': [symbol: string]
}>()

const {
  isConnected,
  isConnecting,
  livePrices: livePricesMap
} = useRealTimeData()

const tradingStore = useTradingStore()

const tickerContainer = ref<HTMLElement>()
const scrollDuration = ref(60) // seconds for full scroll
const recentUpdates = ref<Set<string>>(new Set())

// Convert map to array for easier templating
const livePrices = computed(() => {
  const pricesArray = Array.from(livePricesMap.value.values())

  // Ensure we have at least some default prices
  if (pricesArray.length === 0) {
    return [
      {
        symbol: 'ETH',
        name: 'Ethereum',
        price: 3200,
        change24h: 2.5,
        volume24h: 15000000000,
        marketCap: 385000000000,
        lastUpdated: new Date().toISOString()
      },
      {
        symbol: 'BTC',
        name: 'Bitcoin',
        price: 65000,
        change24h: -1.2,
        volume24h: 25000000000,
        marketCap: 1280000000000,
        lastUpdated: new Date().toISOString()
      },
      {
        symbol: 'USDC',
        name: 'USD Coin',
        price: 1.0,
        change24h: 0.01,
        volume24h: 8000000000,
        marketCap: 32000000000,
        lastUpdated: new Date().toISOString()
      }
    ]
  }

  return pricesArray.slice(0, 10) // Limit to 10 tokens for performance
})

// Connection status
const connectionStatusClass = computed(() => {
  if (isConnected.value) return 'bg-green-500/20 text-green-400 border border-green-500/30'
  if (isConnecting.value) return 'bg-yellow-500/20 text-yellow-400 border border-yellow-500/30'
  return 'bg-red-500/20 text-red-400 border border-red-500/30'
})

const statusDotClass = computed(() => {
  if (isConnected.value) return 'bg-green-400 animate-pulse'
  if (isConnecting.value) return 'bg-yellow-400 animate-pulse'
  return 'bg-red-400'
})

const connectionStatusText = computed(() => {
  if (isConnected.value) return 'LIVE'
  if (isConnecting.value) return 'CONNECTING'
  return 'OFFLINE'
})

// Methods
function formatPrice(price: number): string {
  if (price >= 1000) return price.toFixed(0)
  if (price >= 1) return price.toFixed(2)
  return price.toFixed(4)
}

function getTokenIcon(symbol: string): string {
  const iconMap: Record<string, string> = {
    ETH: '/tokens/eth.svg',
    BTC: '/tokens/btc.svg',
    USDC: '/tokens/usdc.svg',
    USDT: '/tokens/usdt.svg',
    DAI: '/tokens/dai.svg',
    UNI: '/tokens/uni.svg',
    AAVE: '/tokens/aave.svg'
  }
  return iconMap[symbol] || '/tokens/default.svg'
}

function handleImageError(event: Event) {
  const img = event.target as HTMLImageElement
  img.src = '/tokens/default.svg'
}

function selectToken(symbol: string) {
  emit('token-selected', symbol)

  // If we're on a trading page, auto-select this token
  if (tradingStore.tokenIn && !tradingStore.tokenOut) {
    // Set as output token if input is already selected
    const token = tradingStore.popularTokens.find(t => t.symbol === symbol)
    if (token) {
      tradingStore.setTokenOut(token.address)
    }
  }
}

function isRecentUpdate(price: TokenPrice): boolean {
  const updateTime = new Date(price.lastUpdated).getTime()
  const now = Date.now()
  return now - updateTime < 5000 // 5 seconds
}

// Watch for price updates to highlight recent changes
watch(livePricesMap, (newPrices, oldPrices) => {
  if (!oldPrices) return

  for (const [symbol, newPrice] of newPrices) {
    const oldPrice = oldPrices.get(symbol)
    if (!oldPrice || oldPrice.lastUpdated !== newPrice.lastUpdated) {
      // Mark as recently updated
      recentUpdates.value.add(symbol)

      // Remove after 3 seconds
      setTimeout(() => {
        recentUpdates.value.delete(symbol)
      }, 3000)
    }
  }
}, { deep: true })

// Adjust scroll speed based on content width
function adjustScrollSpeed() {
  if (!tickerContainer.value) return

  const containerWidth = tickerContainer.value.scrollWidth / 2 // Divide by 2 since we duplicate content
  const screenWidth = window.innerWidth

  // Calculate duration based on content width (roughly 100px per second)
  scrollDuration.value = Math.max(30, containerWidth / 100)
}

onMounted(() => {
  adjustScrollSpeed()
  window.addEventListener('resize', adjustScrollSpeed)
})

onUnmounted(() => {
  window.removeEventListener('resize', adjustScrollSpeed)
})

// Recalculate when prices change
watch(livePrices, adjustScrollSpeed)
</script>

<style scoped>
/* Continuous scrolling animation */
@keyframes scroll {
  0% {
    transform: translateX(0);
  }
  100% {
    transform: translateX(-50%);
  }
}

.animate-scroll {
  animation: scroll linear infinite;
}

/* Hide scrollbar on mobile */
.scrollbar-hide {
  -ms-overflow-style: none;
  scrollbar-width: none;
}

.scrollbar-hide::-webkit-scrollbar {
  display: none;
}

/* Pause animation on hover */
.animate-scroll:hover {
  animation-play-state: paused;
}
</style>