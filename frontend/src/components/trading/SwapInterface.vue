<template>
  <Card variant="glass" size="lg" class="w-full max-w-md mx-auto">
    <template #header>
      <div class="flex items-center justify-between w-full">
        <h2 class="text-xl font-bold text-white">Swap</h2>
        <div class="flex items-center space-x-2">
          <!-- Settings Button -->
          <button
            @click="showSettings = true"
            class="p-2 hover:bg-white/10 rounded-lg transition-colors"
            aria-label="Swap settings"
          >
            <HeroIcon name="Cog6ToothIcon" class="w-5 h-5 text-white/70" />
          </button>

          <!-- Refresh Button -->
          <button
            @click="refreshQuote"
            :disabled="isLoading || !canRefresh"
            class="p-2 hover:bg-white/10 rounded-lg transition-colors disabled:opacity-50"
            :class="{ 'animate-spin': isLoading }"
            aria-label="Refresh quote"
          >
            <HeroIcon name="ArrowPathIcon" class="w-5 h-5 text-white/70" />
          </button>
        </div>
      </div>
    </template>

    <div class="space-y-4">
      <!-- Token Input Section -->
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <label class="text-sm font-medium text-white/80">From</label>
          <div v-if="tokenInBalance" class="text-xs text-white/60">
            Balance: {{ formatBalance(tokenInBalance) }}
            <button
              @click="setMaxAmount"
              class="ml-1 text-moby-400 hover:text-moby-300 underline"
            >
              MAX
            </button>
          </div>
        </div>

        <div class="relative bg-slate-800/30 rounded-xl border border-slate-600/30 hover:border-slate-500/50 transition-colors">
          <div class="flex items-center">
            <div class="flex-1">
              <Input
                v-model="amountIn"
                type="number"
                placeholder="0.0"
                :disabled="!tokenIn || isLoading"
                class="border-0 bg-transparent text-2xl font-semibold"
                @input="handleAmountChange"
              />
            </div>
            <div class="p-2">
              <TokenSelector
                v-model="tokenIn"
                :exclude-token="tokenOut"
                @token-selected="handleTokenInSelected"
              />
            </div>
          </div>
        </div>
      </div>

      <!-- Swap Direction Button -->
      <div class="flex justify-center">
        <button
          @click="swapTokens"
          :disabled="isLoading"
          class="p-3 bg-slate-800/50 hover:bg-slate-700/50 border border-slate-600/50 hover:border-slate-500/50 rounded-xl transition-all group disabled:opacity-50"
          aria-label="Swap token positions"
        >
          <HeroIcon
            name="ArrowsUpDownIcon"
            class="w-5 h-5 text-white/70 group-hover:text-white transition-colors"
          />
        </button>
      </div>

      <!-- Token Output Section -->
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <label class="text-sm font-medium text-white/80">To</label>
          <div v-if="tokenOutBalance" class="text-xs text-white/60">
            Balance: {{ formatBalance(tokenOutBalance) }}
          </div>
        </div>

        <div class="relative bg-slate-800/30 rounded-xl border border-slate-600/30 hover:border-slate-500/50 transition-colors">
          <div class="flex items-center">
            <div class="flex-1">
              <Input
                :model-value="amountOut"
                type="number"
                placeholder="0.0"
                readonly
                :disabled="!tokenOut"
                class="border-0 bg-transparent text-2xl font-semibold"
              />
            </div>
            <div class="p-2">
              <TokenSelector
                v-model="tokenOut"
                :exclude-token="tokenIn"
                @token-selected="handleTokenOutSelected"
              />
            </div>
          </div>
        </div>
      </div>

      <!-- Quote Information -->
      <div v-if="currentQuote && amountIn && amountOut" class="space-y-3 p-4 bg-slate-800/30 rounded-xl">
        <div class="flex items-center justify-between">
          <span class="text-sm text-white/60">Rate</span>
          <span class="text-sm text-white">
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
          <span class="text-sm text-white/60">Est. Gas Fee</span>
          <span class="text-sm text-white">${{ formatGasFee(gasEstimate) }}</span>
        </div>

        <div class="flex items-center justify-between">
          <span class="text-sm text-white/60">Slippage Tolerance</span>
          <span class="text-sm text-white">{{ defaultSlippage }}%</span>
        </div>

        <!-- Route Information -->
        <div v-if="currentQuote.route.length > 0" class="pt-2 border-t border-white/10">
          <div class="text-xs text-white/60 mb-2">Route</div>
          <div class="space-y-1">
            <div
              v-for="(route, index) in currentQuote.route"
              :key="index"
              class="flex items-center justify-between text-xs"
            >
              <span class="text-white/80">{{ route.dex }}</span>
              <span class="text-white/60">{{ route.percentage }}%</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Wallet Connection / Swap Button -->
      <div class="space-y-3">
        <div v-if="!isConnected">
          <WalletConnector />
        </div>
        <div v-else-if="needsNetworkSwitch">
          <Button
            @click="switchToMainnet"
            variant="warning"
            size="lg"
            full
            :loading="isSwitchingNetwork"
          >
            Switch to Supported Network
          </Button>
        </div>
        <div v-else>
          <Button
            @click="executeSwap"
            :disabled="!isValidTrade || isLoading"
            :loading="isExecuting"
            variant="whale"
            size="lg"
            full
          >
            <template v-if="!tokenIn || !tokenOut">
              Select Tokens
            </template>
            <template v-else-if="!amountIn || parseFloat(amountIn) <= 0">
              Enter Amount
            </template>
            <template v-else-if="insufficientBalance">
              Insufficient {{ getTokenSymbol(tokenIn) }} Balance
            </template>
            <template v-else>
              Swap {{ getTokenSymbol(tokenIn) }} for {{ getTokenSymbol(tokenOut) }}
            </template>
          </Button>
        </div>
      </div>
    </div>

    <!-- Settings Modal -->
    <SwapSettings
      v-if="showSettings"
      @close="showSettings = false"
    />
  </Card>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { storeToRefs } from 'pinia'

import Card from '@components/ui/Card.vue'
import Input from '@components/ui/Input.vue'
import Button from '@components/ui/Button.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'
import TokenSelector from './TokenSelector.vue'
import WalletConnector from '@components/wallet/WalletConnector.vue'
import SwapSettings from './SwapSettings.vue'

import { useTradingStore } from '@/stores/trading'
import { useWallet } from '@/composables/useWallet'
import { useNotificationStore } from '@/stores/notifications'
import type { TokenInfo } from '@/types'

const tradingStore = useTradingStore()
const notificationStore = useNotificationStore()

const {
  tokenIn,
  tokenOut,
  amountIn,
  amountOut,
  priceImpact,
  gasEstimate,
  defaultSlippage,
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
const isExecuting = ref(false)
const isSwitchingNetwork = ref(false)

// Mock balances (would come from wallet in real app)
const tokenBalances = ref<Record<string, string>>({
  '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2': '2.5432', // WETH
  '0xA0b86a33E6847d7b1e7bCd15F6FdE5d2b9FC1234': '1250.00', // USDC
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

const canRefresh = computed(() => {
  return tokenIn.value && tokenOut.value && amountIn.value && parseFloat(amountIn.value) > 0
})

const exchangeRate = computed(() => {
  if (!amountIn.value || !amountOut.value) return '0'
  const rate = parseFloat(amountOut.value) / parseFloat(amountIn.value)
  return rate.toFixed(6)
})

// Methods
function handleAmountChange(value: string) {
  tradingStore.setAmountIn(value)

  if (value && parseFloat(value) > 0 && tokenIn.value && tokenOut.value) {
    // Debounce quote fetching
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

function handleTokenInSelected(token: TokenInfo) {
  tradingStore.setTokenIn(token.address)
  if (amountIn.value && tokenOut.value) {
    debounceQuote()
  }
}

function handleTokenOutSelected(token: TokenInfo) {
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
  if (canRefresh.value) {
    tradingStore.fetchQuote(tokenIn.value!, tokenOut.value!, amountIn.value!)
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
      slippageTolerance: defaultSlippage.value,
      deadline: 20, // 20 minutes
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

    // Simulate trade progression
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
    notificationStore.notifySystem(
      'Swap Failed',
      `Failed to execute swap: ${message}`,
      'error'
    )
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
  const gasPrice = 20 // 20 gwei
  const gasCost = (parseFloat(gasEstimate) * gasPrice * 1e-9) * 3200 // Assuming ETH price $3200
  return gasCost.toFixed(2)
}

// Initialize with default tokens
onMounted(() => {
  tradingStore.initializeDefaultTokens()
})

// Watch for token changes to fetch quotes
watch([tokenIn, tokenOut, amountIn], () => {
  if (tokenIn.value && tokenOut.value && amountIn.value && parseFloat(amountIn.value) > 0) {
    debounceQuote()
  }
})
</script>