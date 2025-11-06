<template>
  <div class="min-h-screen bg-gradient-to-br from-slate-900 via-blue-900 to-slate-900 pb-safe">
    <!-- Mobile Header -->
    <div class="sticky top-0 z-40 bg-slate-900/95 backdrop-blur-xl border-b border-white/10">
      <div class="flex items-center justify-between px-4 py-3">
        <button
          @click="$emit('back')"
          class="p-2 -ml-2 hover:bg-white/10 rounded-lg transition-colors"
          aria-label="Go back"
        >
          <HeroIcon name="ArrowLeftIcon" class="w-6 h-6 text-white" />
        </button>

        <h1 class="text-lg font-bold text-white">Swap</h1>

        <div class="flex items-center space-x-2">
          <button
            @click="showSettings = true"
            class="p-2 hover:bg-white/10 rounded-lg transition-colors"
            aria-label="Settings"
          >
            <HeroIcon name="Cog6ToothIcon" class="w-5 h-5 text-white/70" />
          </button>

          <button
            @click="refreshQuote"
            :disabled="isLoading"
            class="p-2 hover:bg-white/10 rounded-lg transition-colors disabled:opacity-50"
            :class="{ 'animate-spin': isLoading }"
            aria-label="Refresh"
          >
            <HeroIcon name="ArrowPathIcon" class="w-5 h-5 text-white/70" />
          </button>
        </div>
      </div>
    </div>

    <!-- Mobile Swap Content -->
    <div class="p-4 space-y-6">
      <!-- Wallet Status Card -->
      <Card v-if="!isConnected" variant="glass" class="text-center p-6">
        <div class="space-y-4">
          <div class="w-16 h-16 bg-moby-500/20 rounded-full flex items-center justify-center mx-auto">
            <HeroIcon name="WalletIcon" class="w-8 h-8 text-moby-400" />
          </div>
          <div>
            <h3 class="text-lg font-semibold text-white mb-2">Connect Your Wallet</h3>
            <p class="text-white/60 text-sm mb-4">
              Connect your wallet to start trading on Moby Market
            </p>
            <WalletConnector />
          </div>
        </div>
      </Card>

      <!-- Network Warning -->
      <Card v-if="isConnected && needsNetworkSwitch" variant="glass" class="border-yellow-500/30">
        <div class="flex items-start space-x-3 p-4">
          <HeroIcon name="ExclamationTriangleIcon" class="w-6 h-6 text-yellow-400 flex-shrink-0 mt-0.5" />
          <div class="flex-1">
            <h4 class="font-medium text-white">Unsupported Network</h4>
            <p class="text-white/60 text-sm mt-1 mb-3">
              Please switch to a supported network to continue trading.
            </p>
            <Button @click="switchToMainnet" variant="warning" size="sm" full>
              Switch Network
            </Button>
          </div>
        </div>
      </Card>

      <!-- Mobile Swap Interface -->
      <div v-if="isConnected && !needsNetworkSwitch" class="space-y-4">
        <!-- From Token -->
        <Card variant="glass" padding="none">
          <div class="p-4 pb-2">
            <div class="flex items-center justify-between mb-3">
              <label class="text-sm font-medium text-white/80">From</label>
              <div v-if="tokenInBalance" class="text-xs text-white/60">
                Balance: {{ formatBalance(tokenInBalance) }}
              </div>
            </div>
          </div>

          <div class="px-4 pb-4">
            <!-- Amount Input -->
            <div class="mb-4">
              <div class="relative">
                <input
                  v-model="amountIn"
                  type="number"
                  placeholder="0.0"
                  class="w-full bg-transparent text-3xl font-bold text-white placeholder-white/40 focus:outline-none"
                  @input="handleAmountChange"
                />
                <button
                  v-if="tokenInBalance"
                  @click="setMaxAmount"
                  class="absolute right-0 top-1/2 transform -translate-y-1/2 text-moby-400 text-sm font-medium px-2 py-1 rounded hover:bg-moby-500/10"
                >
                  MAX
                </button>
              </div>
            </div>

            <!-- Token Selector -->
            <MobileTokenSelector
              v-model="tokenIn"
              :exclude-token="tokenOut"
              @token-selected="handleTokenInSelected"
            />
          </div>
        </Card>

        <!-- Swap Direction -->
        <div class="flex justify-center">
          <button
            @click="swapTokens"
            class="p-4 bg-slate-800/80 backdrop-blur-sm border border-slate-600/50 rounded-2xl hover:bg-slate-700/80 transition-all active:scale-95"
            aria-label="Swap tokens"
          >
            <HeroIcon name="ArrowsUpDownIcon" class="w-6 h-6 text-white" />
          </button>
        </div>

        <!-- To Token -->
        <Card variant="glass" padding="none">
          <div class="p-4 pb-2">
            <div class="flex items-center justify-between mb-3">
              <label class="text-sm font-medium text-white/80">To</label>
              <div v-if="tokenOutBalance" class="text-xs text-white/60">
                Balance: {{ formatBalance(tokenOutBalance) }}
              </div>
            </div>
          </div>

          <div class="px-4 pb-4">
            <!-- Output Amount -->
            <div class="mb-4">
              <div class="relative">
                <input
                  :value="amountOut"
                  type="number"
                  placeholder="0.0"
                  readonly
                  class="w-full bg-transparent text-3xl font-bold text-white placeholder-white/40 focus:outline-none"
                />
                <div v-if="isLoading" class="absolute right-0 top-1/2 transform -translate-y-1/2">
                  <div class="animate-spin rounded-full h-6 w-6 border-2 border-white/20 border-t-white"></div>
                </div>
              </div>
            </div>

            <!-- Token Selector -->
            <MobileTokenSelector
              v-model="tokenOut"
              :exclude-token="tokenIn"
              @token-selected="handleTokenOutSelected"
            />
          </div>
        </Card>

        <!-- Quote Details -->
        <Card v-if="currentQuote && amountIn && amountOut" variant="glass" padding="sm">
          <div class="space-y-3">
            <div class="flex items-center justify-between">
              <span class="text-sm text-white/60">Rate</span>
              <span class="text-sm text-white font-medium">
                1 {{ getTokenSymbol(tokenIn) }} = {{ exchangeRate }} {{ getTokenSymbol(tokenOut) }}
              </span>
            </div>

            <div class="flex items-center justify-between">
              <span class="text-sm text-white/60">Price Impact</span>
              <span :class="[
                'text-sm font-medium',
                priceImpact > 5 ? 'text-red-400' :
                priceImpact > 2 ? 'text-yellow-400' : 'text-green-400'
              ]">
                {{ priceImpact.toFixed(2) }}%
              </span>
            </div>

            <div class="flex items-center justify-between">
              <span class="text-sm text-white/60">Network Fee</span>
              <span class="text-sm text-white">${{ formatGasFee(gasEstimate) }}</span>
            </div>

            <!-- Expandable Route Info -->
            <button
              v-if="currentQuote.route.length > 0"
              @click="showRouteDetails = !showRouteDetails"
              class="w-full flex items-center justify-between py-2 text-sm text-white/60 hover:text-white transition-colors"
            >
              <span>Route via {{ currentQuote.route.length }} DEX{{ currentQuote.route.length > 1 ? 'es' : '' }}</span>
              <HeroIcon
                name="ChevronDownIcon"
                class="w-4 h-4 transition-transform"
                :class="{ 'rotate-180': showRouteDetails }"
              />
            </button>

            <Transition
              name="route-expand"
              enter-active-class="transition-all duration-200"
              enter-from-class="max-h-0 opacity-0"
              enter-to-class="max-h-32 opacity-100"
              leave-active-class="transition-all duration-200"
              leave-from-class="max-h-32 opacity-100"
              leave-to-class="max-h-0 opacity-0"
            >
              <div v-if="showRouteDetails" class="space-y-2 pt-2 border-t border-white/10 overflow-hidden">
                <div
                  v-for="(route, index) in currentQuote.route"
                  :key="index"
                  class="flex items-center justify-between text-xs"
                >
                  <span class="text-white/70">{{ route.dex }}</span>
                  <span class="text-white/50">{{ route.percentage }}%</span>
                </div>
              </div>
            </Transition>
          </div>
        </Card>

        <!-- Swap Button -->
        <div class="sticky bottom-0 bg-gradient-to-t from-slate-900 via-slate-900/95 to-transparent pt-6 pb-safe">
          <Button
            @click="executeSwap"
            :disabled="!isValidTrade || isLoading"
            :loading="isExecuting"
            variant="whale"
            size="lg"
            full
            class="h-14 text-lg font-semibold"
          >
            <template v-if="!tokenIn || !tokenOut">
              Select Tokens
            </template>
            <template v-else-if="!amountIn || parseFloat(amountIn) <= 0">
              Enter Amount
            </template>
            <template v-else-if="insufficientBalance">
              Insufficient {{ getTokenSymbol(tokenIn) }}
            </template>
            <template v-else>
              Swap Tokens
            </template>
          </Button>
        </div>
      </div>
    </div>

    <!-- Mobile Settings Modal -->
    <MobileSwapSettings
      v-if="showSettings"
      @close="showSettings = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { storeToRefs } from 'pinia'

import Card from '@components/ui/Card.vue'
import Button from '@components/ui/Button.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'
import WalletConnector from '@components/wallet/WalletConnector.vue'
import MobileTokenSelector from './MobileTokenSelector.vue'
import MobileSwapSettings from './MobileSwapSettings.vue'

import { useTradingStore } from '@/stores/trading'
import { useWallet } from '@/composables/useWallet'
import { useNotificationStore } from '@/stores/notifications'

const emit = defineEmits<{
  back: []
}>()

const tradingStore = useTradingStore()
const notificationStore = useNotificationStore()

const {
  tokenIn,
  tokenOut,
  amountIn,
  amountOut,
  priceImpact,
  gasEstimate,
  currentQuote,
  isValidTrade,
  isLoading,
  popularTokens
} = storeToRefs(tradingStore)

const {
  isConnected,
  needsNetworkSwitch,
  switchToMainnet
} = useWallet()

const showSettings = ref(false)
const showRouteDetails = ref(false)
const isExecuting = ref(false)

// Mock balances
const tokenBalances = ref<Record<string, string>>({
  '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2': '2.5432',
  '0xA0b86a33E6847d7b1e7bCd15F6FdE5d2b9FC1234': '1250.00',
})

// Computed properties
const tokenInBalance = computed(() => {
  return tokenIn.value ? tokenBalances.value[tokenIn.value] : null
})

const tokenOutBalance = computed(() => {
  return tokenOut.value ? tokenBalances.value[tokenOut.value] : null
})

const insufficientBalance = computed(() => {
  if (!tokenInBalance.value || !amountIn.value) return false
  return parseFloat(amountIn.value) > parseFloat(tokenInBalance.value)
})

const exchangeRate = computed(() => {
  if (!amountIn.value || !amountOut.value) return '0'
  const rate = parseFloat(amountOut.value) / parseFloat(amountIn.value)
  return rate.toFixed(6)
})

// Methods (same as desktop interface)
function handleAmountChange(event: Event) {
  const target = event.target as HTMLInputElement
  tradingStore.setAmountIn(target.value)

  if (target.value && parseFloat(target.value) > 0 && tokenIn.value && tokenOut.value) {
    debounceQuote()
  }
}

let quoteTimeout: NodeJS.Timeout
function debounceQuote() {
  clearTimeout(quoteTimeout)
  quoteTimeout = setTimeout(() => {
    if (tokenIn.value && tokenOut.value && amountIn.value) {
      tradingStore.fetchQuote(tokenIn.value, tokenOut.value, amountIn.value)
    }
  }, 500)
}

function handleTokenInSelected(token: any) {
  tradingStore.setTokenIn(token.address)
  if (amountIn.value && tokenOut.value) {
    debounceQuote()
  }
}

function handleTokenOutSelected(token: any) {
  tradingStore.setTokenOut(token.address)
  if (amountIn.value && tokenIn.value) {
    debounceQuote()
  }
}

function swapTokens() {
  tradingStore.swapTokens()
  if (amountIn.value) {
    debounceQuote()
  }
}

function setMaxAmount() {
  if (tokenInBalance.value) {
    tradingStore.setAmountIn(tokenInBalance.value)
    debounceQuote()
  }
}

function refreshQuote() {
  if (tokenIn.value && tokenOut.value && amountIn.value) {
    tradingStore.fetchQuote(tokenIn.value, tokenOut.value, amountIn.value)
  }
}

async function executeSwap() {
  if (!isValidTrade.value || isExecuting.value) return

  isExecuting.value = true

  try {
    const tradeRequest = {
      tokenIn: tokenIn.value!,
      tokenOut: tokenOut.value!,
      amountIn: amountIn.value!,
      slippageTolerance: tradingStore.defaultSlippage,
      deadline: 20,
      usePrivacy: false,
      mevProtection: true
    }

    const result = await tradingStore.executeTrade(tradeRequest)

    notificationStore.notifyTrade({
      status: 'pending',
      tokenIn: getTokenSymbol(tokenIn.value!),
      tokenOut: getTokenSymbol(tokenOut.value!),
      amountIn: amountIn.value!,
      amountOut: amountOut.value!,
      txHash: result.hash
    })

    // Simulate completion
    setTimeout(() => {
      tradingStore.updateTradeStatus(result.id, 'confirmed', result.hash)
      notificationStore.notifyTrade({
        status: 'completed',
        tokenIn: getTokenSymbol(tokenIn.value!),
        tokenOut: getTokenSymbol(tokenOut.value!),
        amountIn: amountIn.value!,
        amountOut: amountOut.value!,
        txHash: result.hash
      })
    }, 5000)

  } catch (error) {
    const message = error instanceof Error ? error.message : 'Unknown error'
    notificationStore.notifySystem('Swap Failed', message, 'error')
  } finally {
    isExecuting.value = false
  }
}

function getTokenSymbol(address: string): string {
  const token = popularTokens.value.find(t => t.address === address)
  return token?.symbol || 'Unknown'
}

function formatBalance(balance: string): string {
  const num = parseFloat(balance)
  if (num >= 1000000) return `${(num / 1000000).toFixed(2)}M`
  if (num >= 1000) return `${(num / 1000).toFixed(2)}K`
  if (num >= 1) return num.toFixed(4)
  return num.toFixed(6)
}

function formatGasFee(gasEstimate: string): string {
  const gasPrice = 20
  const gasCost = (parseFloat(gasEstimate) * gasPrice * 1e-9) * 3200
  return gasCost.toFixed(2)
}
</script>

<style scoped>
/* Safe area support for mobile devices */
.pb-safe {
  padding-bottom: env(safe-area-inset-bottom);
}

/* Route expansion animation */
.route-expand-enter-active,
.route-expand-leave-active {
  transition: all 0.2s ease;
  overflow: hidden;
}

.route-expand-enter-from,
.route-expand-leave-to {
  max-height: 0;
  opacity: 0;
}

.route-expand-enter-to,
.route-expand-leave-from {
  max-height: 8rem;
  opacity: 1;
}
</style>