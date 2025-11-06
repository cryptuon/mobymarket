<template>
  <div class="relative">
    <!-- Token Selection Button -->
    <button
      @click="toggleTokenList"
      :class="buttonClass"
      class="flex items-center justify-between w-full p-4 rounded-xl transition-all duration-200 group"
      :disabled="disabled"
    >
      <div class="flex items-center space-x-3">
        <!-- Token Icon -->
        <div v-if="selectedToken" class="w-8 h-8 rounded-full overflow-hidden bg-white/10">
          <img
            :src="selectedToken.logoURI"
            :alt="selectedToken.symbol"
            class="w-full h-full object-cover"
            @error="handleImageError"
          />
        </div>
        <div v-else class="w-8 h-8 rounded-full bg-white/10 flex items-center justify-center">
          <HeroIcon name="QuestionMarkCircleIcon" class="w-5 h-5 text-white/50" />
        </div>

        <!-- Token Info -->
        <div class="text-left">
          <div class="font-medium text-white">
            {{ selectedToken?.symbol || 'Select Token' }}
          </div>
          <div v-if="selectedToken" class="text-xs text-white/60">
            {{ selectedToken.name }}
          </div>
          <div v-if="balance && selectedToken" class="text-xs text-white/50">
            Balance: {{ formatBalance(balance) }}
          </div>
        </div>
      </div>

      <HeroIcon
        name="ChevronDownIcon"
        class="w-5 h-5 text-white/60 group-hover:text-white transition-colors"
        :class="{ 'rotate-180': isOpen }"
      />
    </button>

    <!-- Token List Modal -->
    <Teleport to="body">
      <Transition
        name="token-modal"
        enter-active-class="transition-opacity duration-300"
        enter-from-class="opacity-0"
        enter-to-class="opacity-100"
        leave-active-class="transition-opacity duration-200"
        leave-from-class="opacity-100"
        leave-to-class="opacity-0"
      >
        <div
          v-if="isOpen"
          class="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4"
          @click="closeTokenList"
        >
          <Transition
            name="token-content"
            enter-active-class="transition-all duration-300 ease-out"
            enter-from-class="transform scale-95 opacity-0"
            enter-to-class="transform scale-100 opacity-100"
            leave-active-class="transition-all duration-200 ease-in"
            leave-from-class="transform scale-100 opacity-100"
            leave-to-class="transform scale-95 opacity-0"
          >
            <Card
              v-if="isOpen"
              variant="glass"
              size="lg"
              class="w-full max-w-md mx-auto max-h-[80vh] overflow-hidden"
              @click.stop
            >
              <template #header>
                <div class="flex items-center justify-between w-full">
                  <h3 class="text-lg font-semibold text-white">Select Token</h3>
                  <button
                    @click="closeTokenList"
                    class="p-2 hover:bg-white/10 rounded-lg transition-colors"
                    aria-label="Close token selector"
                  >
                    <HeroIcon name="XMarkIcon" class="w-5 h-5 text-white/70" />
                  </button>
                </div>
              </template>

              <!-- Search Input -->
              <div class="mb-4">
                <Input
                  v-model="searchQuery"
                  placeholder="Search tokens..."
                  icon-left="MagnifyingGlassIcon"
                  clearable
                  @input="handleSearch"
                />
              </div>

              <!-- Popular Tokens -->
              <div v-if="!searchQuery" class="mb-6">
                <h4 class="text-sm font-medium text-white/80 mb-3">Popular Tokens</h4>
                <div class="grid grid-cols-2 gap-2">
                  <button
                    v-for="token in popularTokens"
                    :key="token.address"
                    @click="selectToken(token)"
                    class="flex items-center space-x-2 p-3 bg-slate-800/50 hover:bg-slate-700/50 border border-slate-600/50 hover:border-slate-500/50 rounded-lg transition-all group"
                  >
                    <img
                      :src="token.logoURI"
                      :alt="token.symbol"
                      class="w-6 h-6 rounded-full"
                      @error="handleImageError"
                    />
                    <span class="text-white font-medium">{{ token.symbol }}</span>
                  </button>
                </div>
              </div>

              <!-- Token List -->
              <div class="space-y-1 max-h-96 overflow-y-auto">
                <div v-if="filteredTokens.length === 0" class="text-center py-8">
                  <HeroIcon name="MagnifyingGlassIcon" class="w-12 h-12 text-white/30 mx-auto mb-2" />
                  <p class="text-white/60">No tokens found</p>
                  <p class="text-white/40 text-sm mt-1">Try adjusting your search</p>
                </div>

                <button
                  v-for="token in filteredTokens"
                  :key="token.address"
                  @click="selectToken(token)"
                  :disabled="token.address === excludeToken"
                  class="w-full flex items-center justify-between p-4 hover:bg-white/5 rounded-lg transition-all group disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <div class="flex items-center space-x-3">
                    <img
                      :src="token.logoURI"
                      :alt="token.symbol"
                      class="w-8 h-8 rounded-full"
                      @error="handleImageError"
                    />
                    <div class="text-left">
                      <div class="font-medium text-white">{{ token.symbol }}</div>
                      <div class="text-sm text-white/60">{{ token.name }}</div>
                    </div>
                  </div>

                  <div v-if="getTokenBalance(token)" class="text-right">
                    <div class="text-sm text-white">{{ formatBalance(getTokenBalance(token)) }}</div>
                    <div class="text-xs text-white/50">Balance</div>
                  </div>
                </button>
              </div>

              <template #footer>
                <div class="text-center">
                  <p class="text-xs text-white/50">
                    Can't find your token?
                    <button class="text-moby-400 hover:text-moby-300 underline ml-1">
                      Import custom token
                    </button>
                  </p>
                </div>
              </template>
            </Card>
          </Transition>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'

import Card from '@components/ui/Card.vue'
import Input from '@components/ui/Input.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'

import { useTradingStore } from '@/stores/trading'
import type { TokenInfo } from '@/types'

interface Props {
  modelValue?: string // Token address
  excludeToken?: string // Token to exclude from selection
  label?: string
  disabled?: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'token-selected': [token: TokenInfo]
}>()

const tradingStore = useTradingStore()
const { popularTokens } = tradingStore

const isOpen = ref(false)
const searchQuery = ref('')

// Mock token balances (would come from wallet/blockchain in real app)
const tokenBalances = ref<Record<string, string>>({
  '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2': '2.5432', // WETH
  '0xA0b86a33E6847d7b1e7bCd15F6FdE5d2b9FC1234': '1250.00', // USDC
  '0xdAC17F958D2ee523a2206206994597C13D831ec7': '500.00', // USDT
  '0x6B175474E89094C44Da98b954EedeAC495271d0F': '750.25', // DAI
})

// Extended token list (would come from API in real app)
const allTokens = ref<TokenInfo[]>([
  ...popularTokens,
  {
    address: '0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984',
    symbol: 'UNI',
    name: 'Uniswap',
    decimals: 18,
    chainId: 1,
    logoURI: '/tokens/uni.svg'
  },
  {
    address: '0x7Fc66500c84A76Ad7e9c93437bFc5Ac33E2DDaE9',
    symbol: 'AAVE',
    name: 'Aave',
    decimals: 18,
    chainId: 1,
    logoURI: '/tokens/aave.svg'
  },
  {
    address: '0xC18360217D8F7Ab5e7c516566761Ea12Ce7F9D72',
    symbol: 'ENS',
    name: 'Ethereum Name Service',
    decimals: 18,
    chainId: 1,
    logoURI: '/tokens/ens.svg'
  }
])

// Computed properties
const selectedToken = computed(() => {
  if (!props.modelValue) return null
  return allTokens.value.find(token => token.address === props.modelValue)
})

const balance = computed(() => {
  if (!selectedToken.value) return null
  return getTokenBalance(selectedToken.value)
})

const buttonClass = computed(() => [
  'bg-slate-800/50 hover:bg-slate-700/50 border border-slate-600/50 hover:border-slate-500/50',
  {
    'opacity-50 cursor-not-allowed': props.disabled
  }
])

const filteredTokens = computed(() => {
  if (!searchQuery.value) return allTokens.value

  const query = searchQuery.value.toLowerCase()
  return allTokens.value.filter(token =>
    token.symbol.toLowerCase().includes(query) ||
    token.name.toLowerCase().includes(query) ||
    token.address.toLowerCase().includes(query)
  )
})

// Methods
function toggleTokenList() {
  if (props.disabled) return
  isOpen.value = !isOpen.value
}

function closeTokenList() {
  isOpen.value = false
  searchQuery.value = ''
}

function selectToken(token: TokenInfo) {
  emit('update:modelValue', token.address)
  emit('token-selected', token)
  closeTokenList()
}

function handleSearch(value: string) {
  searchQuery.value = value
}

function getTokenBalance(token: TokenInfo): string | null {
  return tokenBalances.value[token.address] || null
}

function formatBalance(balance: string | null): string {
  if (!balance) return '0'
  const num = parseFloat(balance)
  if (num >= 1000000) return `${(num / 1000000).toFixed(2)}M`
  if (num >= 1000) return `${(num / 1000).toFixed(2)}K`
  if (num >= 1) return num.toFixed(4)
  return num.toFixed(6)
}

function handleImageError(event: Event) {
  const img = event.target as HTMLImageElement
  img.src = '/tokens/default.svg'
}

// Handle escape key
function handleEscape(event: KeyboardEvent) {
  if (event.key === 'Escape' && isOpen.value) {
    closeTokenList()
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleEscape)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleEscape)
})
</script>

<style scoped>
/* Modal animations */
.token-modal-enter-active,
.token-modal-leave-active {
  transition: opacity 0.3s ease;
}

.token-modal-enter-from,
.token-modal-leave-to {
  opacity: 0;
}

.token-content-enter-active {
  transition: all 0.3s ease-out;
}

.token-content-leave-active {
  transition: all 0.2s ease-in;
}

.token-content-enter-from,
.token-content-leave-to {
  transform: scale(0.95);
  opacity: 0;
}

/* Custom scrollbar for token list */
.max-h-96::-webkit-scrollbar {
  width: 4px;
}

.max-h-96::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
}

.max-h-96::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.3);
  border-radius: 2px;
}

.max-h-96::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.5);
}
</style>